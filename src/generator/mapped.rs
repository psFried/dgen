use generator::{GeneratorArg, Generator, DynGenerator, DataGenRng};
use writer::DataGenOutput;
use failure::Error;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::borrow::Borrow;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct Resetter(Rc<(AtomicBool, AtomicBool)>);
impl Resetter {
    fn new() -> Resetter {
        Resetter(Rc::new((AtomicBool::new(true), AtomicBool::new(true))))
    }

    fn reset(&self) {
        (self.0).0.store(true, Ordering::Relaxed);
        (self.0).1.store(true, Ordering::Relaxed);
    }

    fn is_value_reset(&self) -> bool {
        (self.0).0.load(Ordering::Relaxed)
    }

    fn value_set(&self) {
        (self.0).0.store(false, Ordering::Relaxed)
    }

    fn is_bytes_reset(&self) -> bool {
        (self.0).1.load(Ordering::Relaxed)
    }

    fn bytes_set(&self) {
        (self.0).1.store(false, Ordering::Relaxed)
    }
}

// Holy type constraints, Batman!
struct ClosureArgumentInner<R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> {
    name: String,
    wrapped_gen: DynGenerator<R>,
    resetter: Resetter,
    memoized_value: Option<T>,
    memoized_bytes: Vec<u8>,
}

impl <R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> ClosureArgumentInner<R, T> {
    fn new(wrapped_gen: DynGenerator<R>, name: String) -> ClosureArgumentInner<R, T> {
        ClosureArgumentInner {
            name,
            wrapped_gen: wrapped_gen,
            resetter: Resetter::new(),
            memoized_value: None,
            memoized_bytes: Vec::new(),
        }
    }
    fn update_value_if_needed(&mut self, rng: &mut DataGenRng) -> Result<(), Error> {
        if self.resetter.is_value_reset() {
            let ClosureArgumentInner {ref mut wrapped_gen, ref mut memoized_value, ref resetter, ..} = *self;
            let value = wrapped_gen.gen_value(rng)?.map(|v| v.to_owned());
            *memoized_value = value;
            resetter.value_set();
        } 
        Ok(())
    }
    fn write_bytes(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        if self.resetter.is_bytes_reset() {
            let ClosureArgumentInner {ref mut wrapped_gen, ref mut memoized_bytes, ..} = *self;
            memoized_bytes.clear();
            let mut writer = DataGenOutput::new(memoized_bytes);
            wrapped_gen.write_value(rng, &mut writer)?;
            self.resetter.bytes_set();
        } 

        output.write_bytes(self.memoized_bytes.as_slice()).map_err(Into::into)
    }

    fn get_resetter(&self) -> Resetter {
        self.resetter.clone()
    }
}

pub struct ClosureArgument<R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static>(Rc<RefCell<ClosureArgumentInner<R, T>>>);

impl <R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> Clone for ClosureArgument<R, T> {
    fn clone(&self) -> Self {
        let inner = self.0.clone();
        ClosureArgument(inner)
    }
}

pub fn create_arg(name: String, gen: GeneratorArg) -> (GeneratorArg, Resetter) {
    match gen {
        GeneratorArg::Bool(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::Bool(Box::new(arg)), resetter)
        }
        GeneratorArg::Char(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::Char(Box::new(arg)), resetter)
        }
        GeneratorArg::SignedInt(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::SignedInt(Box::new(arg)), resetter)
        }
        GeneratorArg::UnsignedInt(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::UnsignedInt(Box::new(arg)), resetter)
        }
        GeneratorArg::Decimal(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::Decimal(Box::new(arg)), resetter)
        }
        GeneratorArg::String(g) => {
            let arg = ClosureArgument::new(g, name);
            let resetter = arg.get_resetter();
            (GeneratorArg::String(Box::new(arg)), resetter)
        }
    }
}

impl <R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> ClosureArgument<R, T> {
    pub fn new(gen: DynGenerator<R>, name: String) -> ClosureArgument<R, T> {
        let inner = ClosureArgumentInner::new(gen, name);
        ClosureArgument(Rc::new(RefCell::new(inner)))
    }

    fn get_resetter(&self) -> Resetter {
        let inner = (*self.0).borrow();
        inner.get_resetter()
    }
}

impl <R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> Display for ClosureArgument<R, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: Ref<ClosureArgumentInner<R, T>> = (*self.0).borrow();
        write!(f, "{}({})", inner.name, inner.wrapped_gen)
    }
}

impl <R: Display + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> Generator for ClosureArgument<R, T> {
    type Output = R;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&R>, Error> {
        let mut inner = self.0.borrow_mut();
        inner.update_value_if_needed(rng)?;
        let value = &inner.memoized_value as *const Option<T>;
        unsafe {
            let coerced = &*value;
            Ok(coerced.as_ref().map(|v| v.borrow()))
        }
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        let mut inner = self.0.borrow_mut();
        inner.write_bytes(rng, output)
    }

    fn new_from_prototype(&self) -> DynGenerator<R> {
        let inner = self.0.clone();
        Box::new(ClosureArgument(inner))
    }
}


pub struct ArgResettingGen<T: Display + ?Sized + 'static> {
    resetter: Resetter,
    mapper: DynGenerator<T>,
}

impl <T: Display + ?Sized + 'static> ArgResettingGen<T> {
    pub fn new(mapper: DynGenerator<T>, resetter: Resetter) -> DynGenerator<T> {
        Box::new(ArgResettingGen{ mapper, resetter })
    }
}

pub fn wrap_mapped_gen(gen: GeneratorArg, resetter: Resetter) -> GeneratorArg {
    match gen {
        GeneratorArg::UnsignedInt(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::UnsignedInt(wrapped)
        }
        GeneratorArg::SignedInt(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::SignedInt(wrapped)
        }
        GeneratorArg::Decimal(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::Decimal(wrapped)
        }
        GeneratorArg::Char(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::Char(wrapped)
        }
        GeneratorArg::String(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::String(wrapped)
        }
        GeneratorArg::Bool(g) => {
            let wrapped = ArgResettingGen::new(g, resetter);
            GeneratorArg::Bool(wrapped)
        }
    }
}

impl <T: Display + ?Sized + 'static> Display for ArgResettingGen<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mapper)
    }
}

impl <T: Display + ?Sized + 'static> Generator for ArgResettingGen<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        self.resetter.reset();
        self.mapper.gen_value(rng)
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        self.resetter.reset();
        self.mapper.write_value(rng, output)
    }

    fn new_from_prototype(&self) -> DynGenerator<T> {
        unimplemented!();
    }
}

