use crate::object::Registers;

pub trait HasRegisters {
    fn get_registers(&self) -> &Registers;
    fn get_buffer(&self) -> Option<impl HasBuffer>;
}

pub trait HasBuffer: HasRegisters {
    fn get_buffer(&self);
}
