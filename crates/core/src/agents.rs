//! Structure-of-arrays agent storage.
pub mod state {
    pub const S: u8 = 0;
    pub const E: u8 = 1;
    /// Infectious, symptomless.
    pub const A: u8 = 2;
    /// Infectious, symptomatic.
    pub const I: u8 = 3;
    pub const H: u8 = 4;
    pub const R: u8 = 5;
    pub const D: u8 = 6;
    pub const COUNT: usize = 7;
    pub const NAMES: [&str; COUNT] = ["S", "E", "A", "I", "H", "R", "D"];
}

#[derive(Clone, Debug, Default)]
pub struct Agents {
    pub id: Vec<u32>,
    /// 0: 0-17, 1: 18-39, 2: 40-59, 3: 60-74, 4: 75+
    pub age_group: Vec<u8>,
    pub state: Vec<u8>,
    /// Steps remaining in the current state (0 for S/R/D).
    pub timer: Vec<u16>,
    pub home_loc: Vec<u32>,
    /// Workplace or school/college; equals home_loc when none.
    pub work_loc: Vec<u32>,
    /// 0 = unvaccinated, 1 = vaccinated.
    pub vax_status: Vec<u8>,
    /// Days since vaccination.
    pub vax_days: Vec<u16>,
}

impl Agents {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            id: Vec::with_capacity(n),
            age_group: Vec::with_capacity(n),
            state: Vec::with_capacity(n),
            timer: Vec::with_capacity(n),
            home_loc: Vec::with_capacity(n),
            work_loc: Vec::with_capacity(n),
            vax_status: Vec::with_capacity(n),
            vax_days: Vec::with_capacity(n),
        }
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push(&mut self, age_group: u8, home: u32, work: u32, vax_status: u8, vax_days: u16) {
        self.id.push(self.id.len() as u32);
        self.age_group.push(age_group);
        self.state.push(state::S);
        self.timer.push(0);
        self.home_loc.push(home);
        self.work_loc.push(work);
        self.vax_status.push(vax_status);
        self.vax_days.push(vax_days);
    }

    pub fn counts(&self) -> [u32; state::COUNT] {
        let mut c = [0u32; state::COUNT];
        for &s in &self.state {
            c[s as usize] += 1;
        }
        c
    }
}
