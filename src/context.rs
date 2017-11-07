use {Chomp, Error, Path, Result};
use serde_json::Value;
use std::io::Write;

pub trait Context<W> {
    fn inject(&self, path: Path, sink: &mut W) -> Result<()>;
    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()>;
}

impl<'a, W, T: Context<W> + ?Sized> Context<W> for &'a T {
    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        (*self).inject(path, sink)
    }
    fn iterate(&self, path: Path, chomp: Chomp<W>) -> Result<()> {
        (*self).iterate(path, chomp)
    }
}

macro_rules! display_impls {
($($x:ty),*) => {$(
    impl<W: Write> Context<W> for $x {
        fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
            match path.parts().next() {
                Some(_) => Err(Error::Undefined(path.to_owned())),
                None => {
                    write!(sink, "{}", self)?;
                    Ok(())
                },
            }
        }

        fn iterate(&self, path: Path, _: Chomp<W>) -> Result<()> {
            match path.parts().next() {
                Some(_) => Err(Error::Undefined(path.to_owned())),
                None => Err(Error::NotIterable(path.to_owned())),
            }
        }
    }
)*}
}

display_impls! {
    bool,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    f32, f64,
    char, str
}

impl<W: Write> Context<W> for Value {
    fn inject(&self, path: Path, sink: &mut W) -> Result<()> {
        use Value::*;

        let mut value = self;

        for part in path.parts() {
            if let Some(next_value) = value.get(part) {
                value = next_value;
            } else {
                return Err(Error::Undefined(path.to_owned()));
            }
        }

        match value {
            &String(ref s) => sink.write_all(s.as_bytes()),
            x => write!(sink, "{}", x),
        }?;

        Ok(())
    }

    fn iterate(&self, path: Path, mut chomp: Chomp<W>) -> Result<()> {
        let mut value = self;

        for part in path.parts() {
            if let Some(next_value) = value.get(part) {
                value = next_value;
            } else {
                return Err(Error::Undefined(path.to_owned()));
            }
        }

        if let &Value::Array(ref array) = value {
            for value in array {
                chomp.chomp(value)?;
            }
            Ok(())
        } else {
            Err(Error::NotIterable(path.to_owned()))
        }
    }
}