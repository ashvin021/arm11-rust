pub struct ArmMachineState {
    main_memory: [u8; 65535],
    register_file: [u32; 17],
    pipeline: Pipeline,
}

pub struct Pipeline {
    fetched: Instruction,
    decoded: Instruction,
}

pub enum Instruction {
    DataProc,
    Multiply,
    SDT,
    Branch,
}

pub struct DataProc {
    cond: ConditionCode,
    opcode: u8,
    is_immediate: bool,
    set_cond: bool,
    rn: u8,
    rd: u8,
    operand2: Operand2,
}

pub struct Multiply {
    cond: ConditionCode,
    accumulate: bool,
    set_cond: bool,
    rd: u8,
    rn: u8,
    rs: u8,
    rm: u8,
}

pub struct SDT {
    cond: ConditionCode,
    is_shifted_r: bool,
    is_preindexed: bool,
    up_bit: bool,
    load: bool,
    rn: u8,
    rd: u8,
    offset: Operand2,
}

pub struct Branch {
    cond: ConditionCode,
    offset: i32,
}

pub enum Operand2 {
    ConstantShift(u8, u8),
    ConstantShiftedReg(u8, ShiftType, u8),
    ShiftedReg(u8, ShiftType, u8),
}

pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

pub enum ConditionCode {
    Eq_,
    Ne,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
}
