use crate::object::Registers;

pub trait HasRegisters {
    fn get_registers(&self) -> &Registers;
    fn get_buffer(&self) -> Result<impl HasBuffer, ()>;
}

pub trait HasBuffer: HasRegisters {
    fn get_buffer(&self) -> ();
}

