use failure::Error;
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use v2::{AnyFunction, DataGenOutput, DynFun, ProgramContext, RunnableFunction};

pub struct Resetter {
    value: AtomicBool,
    bytes: AtomicBool,
}
impl Resetter {
    fn new() -> Resetter {
        Resetter {
            value: AtomicBool::new(true),
            bytes: AtomicBool::new(true),
        }
    }

    fn reset(&self) {
        self.value.store(true, Ordering::Relaxed);
        self.bytes.store(true, Ordering::Relaxed);
    }

    fn is_value_reset(&self) -> bool {
        self.value.load(Ordering::Relaxed)
    }

    fn value_set(&self) {
        self.value.store(false, Ordering::Relaxed)
    }

    fn is_bytes_reset(&self) -> bool {
        self.bytes.load(Ordering::Relaxed)
    }

    fn bytes_set(&self) {
        self.bytes.store(false, Ordering::Relaxed)
    }
}

struct MemoizedValue<T> {
    value: Option<T>,
    bytes: Option<Vec<u8>>,
}
impl<T> MemoizedValue<T> {
    fn new() -> MemoizedValue<T> {
        MemoizedValue {
            value: None,
            bytes: None,
        }
    }
}

pub struct MemoizedFunction<T> {
    wrapped: DynFun<T>,
    memoized: RefCell<MemoizedValue<T>>,
    resetter: Rc<Resetter>,
}

impl<T: Clone + 'static> MemoizedFunction<T> {
    fn new(wrapped: DynFun<T>, resetter: Rc<Resetter>) -> DynFun<T> {
        Rc::new(MemoizedFunction {
            wrapped,
            resetter,
            memoized: RefCell::new(MemoizedValue::new()),
        })
    }
}

impl<T> Debug for MemoizedFunction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "memoized({:?})", self.wrapped)
    }
}

impl<T: Clone> RunnableFunction<T> for MemoizedFunction<T> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<T, Error> {
        if self.resetter.is_value_reset() {
            let new_value = self.wrapped.gen_value(ctx)?;
            let mut cell = self.memoized.borrow_mut();
            cell.value = Some(new_value);
            self.resetter.value_set();
        }
        // clone the value and return it. This is a safe unwrap because we ensure that the resetter
        // always starts out with `is_value_reset` returning true
        Ok(self.memoized.borrow().value.as_ref().cloned().unwrap())
    }

    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let MemoizedFunction {
            ref wrapped,
            ref memoized,
            ref resetter,
        } = *self;
        let mut cell = memoized.borrow_mut();
        if resetter.is_bytes_reset() {
            if cell.bytes.is_none() {
                cell.bytes = Some(Vec::new());
            }
            let buffer = cell.bytes.as_mut().unwrap();
            buffer.clear();
            let mut new_out = DataGenOutput::new(buffer);
            let _ = wrapped.write_value(ctx, &mut new_out)?;
            resetter.bytes_set();
        }
        let bytes = cell.bytes.as_ref().unwrap();
        out.write(bytes)
    }
}

pub struct WrappedMemoizedFunction<T> {
    resetter: Rc<Resetter>,
    usage: DynFun<T>,
}
impl<T> Debug for WrappedMemoizedFunction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.usage)
    }
}

impl<T> RunnableFunction<T> for WrappedMemoizedFunction<T> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<T, Error> {
        self.resetter.reset();
        self.usage.gen_value(ctx)
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        self.resetter.reset();
        self.usage.write_value(ctx, out)
    }
}
impl<T: 'static> WrappedMemoizedFunction<T> {
    pub fn new(usage: DynFun<T>, resetter: Rc<Resetter>) -> DynFun<T> {
        Rc::new(WrappedMemoizedFunction { resetter, usage })
    }
}

pub fn finish_mapped(resolved: AnyFunction, resetter: Rc<Resetter>) -> AnyFunction {
    match resolved {
        AnyFunction::Char(fun) => AnyFunction::Char(WrappedMemoizedFunction::new(fun, resetter)),
        AnyFunction::String(fun) => {
            AnyFunction::String(WrappedMemoizedFunction::new(fun, resetter))
        }
        AnyFunction::Boolean(fun) => {
            AnyFunction::Boolean(WrappedMemoizedFunction::new(fun, resetter))
        }
        AnyFunction::Decimal(fun) => {
            AnyFunction::Decimal(WrappedMemoizedFunction::new(fun, resetter))
        }
        AnyFunction::Uint(fun) => AnyFunction::Uint(WrappedMemoizedFunction::new(fun, resetter)),
        AnyFunction::Int(fun) => AnyFunction::Int(WrappedMemoizedFunction::new(fun, resetter)),
        AnyFunction::Bin(fun) => AnyFunction::Bin(WrappedMemoizedFunction::new(fun, resetter)),
    }
}

pub fn create_memoized_fun(input: AnyFunction) -> (AnyFunction, Rc<Resetter>) {
    let resetter = Rc::new(Resetter::new());
    let reset_to_return = resetter.clone();

    let fun_to_return = match input {
        AnyFunction::Char(fun) => AnyFunction::Char(MemoizedFunction::new(fun, resetter)),
        AnyFunction::String(fun) => AnyFunction::String(MemoizedFunction::new(fun, resetter)),
        AnyFunction::Boolean(fun) => AnyFunction::Boolean(MemoizedFunction::new(fun, resetter)),
        AnyFunction::Decimal(fun) => AnyFunction::Decimal(MemoizedFunction::new(fun, resetter)),
        AnyFunction::Uint(fun) => AnyFunction::Uint(MemoizedFunction::new(fun, resetter)),
        AnyFunction::Int(fun) => AnyFunction::Int(MemoizedFunction::new(fun, resetter)),
        AnyFunction::Bin(fun) => AnyFunction::Bin(MemoizedFunction::new(fun, resetter)),
    };
    (fun_to_return, reset_to_return)
}

// impl<T> MemoizedFunction<T> {
//     pub fn new(wrapped: DynFun<T>) -> (MemoizedFunction<T>, )
// }
