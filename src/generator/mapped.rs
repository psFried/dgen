use generator::{GeneratorType, GeneratorArg, Generator, DynGenerator, DataGenRng};
use writer::DataGenOutput;
use rand::Rng;
use failure::Error;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::borrow::Borrow;
use interpreter::functions::FunctionCreator;

/*
my_fun(arg1: String, arg2: Uint) = arg1() {str_val => 
    # str_value here is wrapped in ClosureArgument
    arg2 { num_val =>
        concat(
            // all references to 'str_value' are just clones of the argument
            repeat(num_val, trailing_newline(str_val)), 
            repeat(num_val, str_val)
        )
    }
};

repeat(3, my_fun(one_of("foo", "bar"), uint(3, 5)) { str_val =>
    # str_value here is wrapped in ClosureArgument, each reference to it is a clone
    concat(str_val, str_val, str_val)
})
*/

// Holy type constraints, Batman!
struct ClosureArgumentInner<R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> {
    name: String,
    wrapped_gen: DynGenerator<R>,
    usages: usize,
    memoized_value: Option<T>,
    memoized_bytes: Vec<u8>,
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> ClosureArgumentInner<R, T> {
    fn new(wrapped_gen: DynGenerator<R>, name: String) -> ClosureArgumentInner<R, T> {
        ClosureArgumentInner {
            name,
            wrapped_gen: wrapped_gen,
            usages: 0,
            memoized_value: None,
            memoized_bytes: Vec::new(),
        }
    }
    fn update_value_if_needed(&mut self, rng: &mut DataGenRng) -> Result<(), Error> {
        if self.get_usage_count() == 0 {
            let ClosureArgumentInner {ref mut wrapped_gen, ref mut memoized_value, ..} = *self;
            let value = wrapped_gen.gen_value(rng)?.map(|v| v.to_owned());
            *memoized_value = value;
        } 
        self.increment_usages();
        Ok(())
    }
    fn write_bytes(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        if self.get_usage_count() == 0 {
            let ClosureArgumentInner {ref mut wrapped_gen, ref mut memoized_bytes, ..} = *self;
            memoized_bytes.clear();
            let mut writer = DataGenOutput::new(memoized_bytes);
            wrapped_gen.write_value(rng, &mut writer)?;
        } 
        self.increment_usages();

        output.write_bytes(self.memoized_bytes.as_slice()).map_err(Into::into)
    }

    fn get_usage_count(&self) -> usize {
        *self.usages.borrow()
    }
    fn increment_usages(&mut self) {
        self.usages += 1;
    }

    fn forget_memoized(&mut self) {
        self.usages = 0;
    }

    fn new_from_prototype(&self) -> ClosureArgumentInner<R, T> {
        let wrapped = self.wrapped_gen.new_from_prototype();
        let name = self.name.clone();
        ClosureArgumentInner::new(wrapped, name)
    }
}

#[derive(Clone)]
pub struct ClosureArgument<R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static>(Rc<RefCell<ClosureArgumentInner<R, T>>>);

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> ClosureArgument<R, T> {
    pub fn new(gen: DynGenerator<R>, name: String) -> ClosureArgument<R, T> {
        let inner = ClosureArgumentInner::new(gen, name);
        ClosureArgument(Rc::new(RefCell::new(inner)))
    }

    fn copy_for_new_use(&self) -> ClosureArgument<R, T> {
        let inner: Ref<ClosureArgumentInner<R, T>> = (*self.0).borrow();
        ClosureArgument(Rc::new(RefCell::new(inner.new_from_prototype())))
    }

    fn reset(&self) {
        let mut inner = self.0.borrow_mut();
        inner.forget_memoized();
    }
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> Display for ClosureArgument<R, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: Ref<ClosureArgumentInner<R, T>> = (*self.0).borrow();
        write!(f, "{}({})", inner.name, inner.wrapped_gen)
    }
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static> Generator for ClosureArgument<R, T> {
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
        panic!("Internal error: Invalid call to new_from_prototype on a closure argument");
    }
}


pub struct MappedGen<R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static, S: Display + Clone + ?Sized + 'static> {
    input: ClosureArgument<R, T>,
    mapper: DynGenerator<S>,
    create_mapper: Rc<Fn(DynGenerator<R>)->DynGenerator<S> + 'static>,
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static, S: Display + Clone + ?Sized + 'static> MappedGen<R, T, S> {

    pub fn new<F>(outer: DynGenerator<R>, arg_name: String, create_mapper: Rc<Fn(DynGenerator<R>)->DynGenerator<S> + 'static>) -> MappedGen<R, T, S> {
        let input = ClosureArgument::new(outer, arg_name);
        let boxed_input = Box::new(input.clone());
        let mapper = create_mapper(boxed_input);
        MappedGen { input, mapper, create_mapper }
    }
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static, S: Display + Clone + ?Sized + 'static> Generator for MappedGen<R, T, S> {
    type Output = S;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&S>, Error> {
        self.input.reset();
        self.mapper.gen_value(rng)
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        self.input.reset();
        self.mapper.write_value(rng, output)
    }

    fn new_from_prototype(&self) -> DynGenerator<S> {
        let input = self.input.copy_for_new_use();
        let create_mapper = self.create_mapper.clone();
        let mapper = create_mapper(Box::new(input.clone()));
        Box::new(MappedGen {input, create_mapper, mapper })
    }
}

impl <R: Display + Clone + ?Sized + ToOwned<Owned=T> + 'static, T: Borrow<R> + Display + Clone + ?Sized + 'static, S: Display + Clone + ?Sized + 'static>  Display for MappedGen<R, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{{ {} }}", self.input, self.mapper)
    }
}

