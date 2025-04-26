pub type FreqDeck = [u8; 5];

pub const SETTLEMENT_COST: FreqDeck = [1, 1, 1, 1, 0];
pub const ROAD_COST: FreqDeck = [1, 1, 0, 0, 0];
pub const CITY_COST: FreqDeck = [0, 0, 0, 2, 3];
pub const DEVCARD_COST: FreqDeck = [0, 0, 1, 1, 1];

pub fn freqdeck_sub(deck: &mut [u8], other: FreqDeck) {
    for i in 0..other.len() {
        deck[i] -= other[i];
    }
}

pub fn freqdeck_add(deck: &mut [u8], other: FreqDeck) {
    for i in 0..other.len() {
        deck[i] += other[i];
    }
}

pub fn freqdeck_contains(deck: &[u8], subdeck: &FreqDeck) -> bool {
    for i in 0..subdeck.len() {
        if deck[i] < subdeck[i] {
            return false;
        }
    }
    true
}
