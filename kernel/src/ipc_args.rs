use crate::object::Registers;

pub trait HasRegisters {
    fn get_registers(&self) -> &Registers;
    fn get_bufferalbe(&self) -> Option<&dyn HasBuffer>;
}

pub trait HasBuffer: HasRegisters {
    fn get_buffer(&self);
}
