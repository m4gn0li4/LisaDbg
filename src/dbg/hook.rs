#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ModIntpr {
    Address,
    Name,
}

impl Default for ModIntpr {
    fn default() -> Self {
        ModIntpr::Name
    }
}




#[derive(Default, Copy, Clone)]
pub struct Hook {
    pub target: u64,
    pub replacen: u64,
}


pub static mut HOOK_FUNC: Vec<Hook> = Vec::new();