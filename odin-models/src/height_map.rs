pub const HEIGHT_BLOCKED: u8 = 127;
pub const MAX_HEIGHT_DIFF: u8 = 8;

#[derive(Debug, Clone)]
pub struct HeightMap {
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl HeightMap {
    pub fn new(width: u16, height: u16, data: Vec<u8>) -> Result<Self, HeightMapError> {
        let expected = width as usize * height as usize;
        if data.len() != expected {
            return Err(HeightMapError::InvalidDataLength {
                expected,
                actual: data.len(),
            });
        }
        Ok(Self {
            width,
            height,
            data,
        })
    }

    pub fn from_raw(bytes: &[u8]) -> Result<Self, HeightMapError> {
        if bytes.len() < 4 {
            return Err(HeightMapError::InvalidHeader);
        }
        let width = u16::from_le_bytes([bytes[0], bytes[1]]);
        let height = u16::from_le_bytes([bytes[2], bytes[3]]);
        let data = bytes[4..].to_vec();
        Self::new(width, height, data)
    }

    pub fn empty(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            data: vec![0; width as usize * height as usize],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn get(&self, x: u16, y: u16) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.data[y as usize * self.width as usize + x as usize])
    }

    pub fn set(&mut self, x: u16, y: u16, value: u8) {
        if x < self.width && y < self.height {
            self.data[y as usize * self.width as usize + x as usize] = value;
        }
    }

    pub fn is_blocked(&self, x: u16, y: u16) -> bool {
        !matches!(self.get(x, y), Some(h) if h != HEIGHT_BLOCKED)
    }

    pub fn can_walk(&self, from_x: u16, from_y: u16, to_x: u16, to_y: u16) -> bool {
        let Some(from_h) = self.get(from_x, from_y) else {
            return false;
        };
        let Some(to_h) = self.get(to_x, to_y) else {
            return false;
        };
        if to_h == HEIGHT_BLOCKED {
            return false;
        }
        let diff = (from_h as i16 - to_h as i16).unsigned_abs() as u8;
        diff <= MAX_HEIGHT_DIFF
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HeightMapError {
    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
    #[error("Invalid header: not enough bytes")]
    InvalidHeader,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_all_walkable() {
        let hm = HeightMap::empty(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                assert!(!hm.is_blocked(x, y));
                assert_eq!(hm.get(x, y), Some(0));
            }
        }
    }

    #[test]
    fn blocked_cell() {
        let mut hm = HeightMap::empty(10, 10);
        hm.set(5, 5, HEIGHT_BLOCKED);
        assert!(hm.is_blocked(5, 5));
    }

    #[test]
    fn can_walk_flat_terrain() {
        let mut hm = HeightMap::empty(10, 10);
        hm.set(0, 0, 50);
        hm.set(1, 0, 50);
        assert!(hm.can_walk(0, 0, 1, 0));
    }

    #[test]
    fn can_walk_steep_terrain() {
        let mut hm = HeightMap::empty(10, 10);
        hm.set(0, 0, 50);
        hm.set(1, 0, 60);
        assert!(!hm.can_walk(0, 0, 1, 0));
    }

    #[test]
    fn can_walk_boundary_diff() {
        let mut hm = HeightMap::empty(10, 10);
        hm.set(0, 0, 50);
        hm.set(1, 0, 58);
        assert!(hm.can_walk(0, 0, 1, 0));
    }

    #[test]
    fn can_walk_to_blocked() {
        let mut hm = HeightMap::empty(10, 10);
        hm.set(0, 0, 50);
        hm.set(1, 0, HEIGHT_BLOCKED);
        assert!(!hm.can_walk(0, 0, 1, 0));
    }

    #[test]
    fn out_of_bounds_is_blocked() {
        let hm = HeightMap::empty(10, 10);
        assert_eq!(hm.get(9999, 9999), None);
        assert!(hm.is_blocked(9999, 9999));
    }

    #[test]
    fn from_raw_roundtrip() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&5u16.to_le_bytes());
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend(vec![42u8; 15]);

        let hm = HeightMap::from_raw(&bytes).unwrap();
        assert_eq!(hm.width(), 5);
        assert_eq!(hm.height(), 3);
        assert_eq!(hm.get(0, 0), Some(42));
        assert_eq!(hm.get(4, 2), Some(42));
    }

    #[test]
    fn new_rejects_wrong_data_length() {
        assert!(HeightMap::new(10, 10, vec![0; 50]).is_err());
    }
}
