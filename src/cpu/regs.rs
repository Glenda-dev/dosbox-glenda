#[derive(Debug, Default, Clone, Copy)]
pub struct Registers {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub si: u16,
    pub di: u16,
    pub sp: u16,
    pub bp: u16,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub ip: u16,
    pub flags: u16,
}
