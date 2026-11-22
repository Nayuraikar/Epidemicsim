//! Philox4x32-10 counter-based RNG: stateless, keyed by (seed, agent, step, purpose).
const M0: u32 = 0xD251_1F53;
const M1: u32 = 0xCD9E_8D57;
const W0: u32 = 0x9E37_79B9;
const W1: u32 = 0xBB67_AE85;

/// Stream identifiers so independent decisions never share a counter.
pub mod purpose {
    pub const INFECT: u32 = 1;
    pub const TRANSITION: u32 = 2;
    pub const SCHEDULE: u32 = 3;
    pub const SEED_INFECT: u32 = 4;
    pub const SYNTH_HH: u32 = 5;
    pub const SYNTH_AGENT: u32 = 6;
    pub const SYNTH_VAX: u32 = 7;
    pub const ISOLATE: u32 = 8;
    pub const VACCINATE: u32 = 9;
    /// Inter-city travel: does an infectious agent travel today, and to which city.
    pub const TRAVEL: u32 = 10;
    /// Inter-city travel: which agent of the destination city an arriving traveller meets.
    pub const TRAVEL_LAND: u32 = 11;
}

#[inline]
fn mulhilo(a: u32, b: u32) -> (u32, u32) {
    let p = u64::from(a) * u64::from(b);
    ((p >> 32) as u32, p as u32)
}

/// Raw Philox4x32 with 10 rounds.
pub fn philox4x32_10(mut ctr: [u32; 4], mut key: [u32; 2]) -> [u32; 4] {
    for r in 0..10 {
        if r > 0 {
            key[0] = key[0].wrapping_add(W0);
            key[1] = key[1].wrapping_add(W1);
        }
        let (hi0, lo0) = mulhilo(M0, ctr[0]);
        let (hi1, lo1) = mulhilo(M1, ctr[2]);
        ctr = [hi1 ^ ctr[1] ^ key[0], lo1, hi0 ^ ctr[3] ^ key[1], lo0];
    }
    ctr
}

/// Map a u32 to a uniform f64 in the open interval (0, 1).
#[inline]
pub fn to_unit(x: u32) -> f64 {
    (f64::from(x) + 0.5) * (1.0 / 4_294_967_296.0)
}

#[derive(Clone, Copy, Debug)]
pub struct Philox {
    key: [u32; 2],
}

impl Philox {
    pub fn new(seed: u64) -> Self {
        Self {
            key: [seed as u32, (seed >> 32) as u32],
        }
    }

    /// Four random u32 lanes for (agent_id, step, purpose).
    #[inline]
    pub fn block(&self, agent_id: u32, step: u32, purpose: u32) -> [u32; 4] {
        philox4x32_10([agent_id, step, purpose, 0], self.key)
    }

    /// Four uniforms in (0, 1) for (agent_id, step, purpose).
    #[inline]
    pub fn uniforms(&self, agent_id: u32, step: u32, purpose: u32) -> [f64; 4] {
        self.block(agent_id, step, purpose).map(to_unit)
    }

    /// One uniform in (0, 1).
    #[inline]
    pub fn uniform(&self, agent_id: u32, step: u32, purpose: u32) -> f64 {
        to_unit(self.block(agent_id, step, purpose)[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-answer vectors from the Random123 distribution (kat_vectors, philox4x32 10).
    #[test]
    fn known_vectors() {
        assert_eq!(
            philox4x32_10([0, 0, 0, 0], [0, 0]),
            [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8]
        );
        assert_eq!(
            philox4x32_10([u32::MAX; 4], [u32::MAX; 2]),
            [0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd]
        );
        assert_eq!(
            philox4x32_10(
                [0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344],
                [0xa4093822, 0x299f31d0]
            ),
            [0xd16cfe09, 0x94fdcceb, 0x5001e420, 0x24126ea1]
        );
    }

    #[test]
    fn uniform_in_open_unit_interval_and_deterministic() {
        let p = Philox::new(42);
        for a in 0..1000 {
            let u = p.uniform(a, 3, purpose::INFECT);
            assert!(u > 0.0 && u < 1.0);
            assert_eq!(u, p.uniform(a, 3, purpose::INFECT));
        }
        assert_ne!(p.block(1, 2, 3), p.block(1, 2, 4));
    }
}
