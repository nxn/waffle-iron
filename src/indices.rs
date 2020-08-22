use std::collections::HashSet;

lazy_static! {
    static ref INDEX_MAP: IndexMap = IndexMap::new();
}

struct IndexMap {
    pub rows:       [ [ usize; 9 ]; 9 ],
    pub columns:    [ [ usize; 9 ]; 9 ],
    pub boxes:      [ [ usize; 9 ]; 9 ],
    pub rcb:        [ HashSet<usize>; 81 ]
}

impl IndexMap {
    pub fn new() -> IndexMap {
        let mut map = IndexMap {
            rows:       [[0; 9]; 9],
            columns:    [[0; 9]; 9],
            boxes:      [[0; 9]; 9],
            rcb:        arr![HashSet::new(); 81]
        };

        map.populate_indices();
        map.generate_rcb();

        map
    }

    fn populate_indices(&mut self) {
        let mut cell_index = 0;

        for y in 0..9 {
            for x in 0..9 {
                // Box index at this x and y
                let b = ((y / 3) * 3) + (x / 3);
                // Cell index (within box)
                let c = ((y % 3) * 3) + (x % 3);

                self.rows[y][x]     = cell_index;
                self.columns[x][y]  = cell_index;
                self.boxes[b][c]    = cell_index;

                cell_index += 1;
            }
        }
    }

    fn generate_rcb(&mut self) {
        for position in 0..81 {
            // Each RCB set contains exactly 21 members (ex: 9 per box, 6 row members extending beyond box, and 6 col
            // members also extending outside box)
            let mut set = HashSet::with_capacity(21);

            let r = self.rows   [ row_index(position) ];
            let c = self.columns[ col_index(position) ];
            let b = self.boxes  [ box_index(position) ];

            // insert all members of the row, column, and box into the set
            for i in 0..9 {
                set.insert(r[i]);
                set.insert(c[i]);
                set.insert(b[i]);
            }

            self.rcb[position] = set;
        }
    }
}

#[inline]
pub fn row_index(cell_index: usize) -> usize {
    cell_index / 9
}

#[inline]
pub fn col_index(cell_index: usize) -> usize {
    cell_index % 9
}

#[inline]
pub fn box_index(cell_index: usize) -> usize {
    let box_y = cell_index / 27 * 3;
    let box_x = cell_index / 3  % 3;
    
    box_x + box_y
}

#[inline]
pub fn row_at(row_index: usize) -> [usize; 9] {
    INDEX_MAP.rows[ row_index ]
}

#[inline]
pub fn col_at(col_index: usize) -> [usize; 9] {
    INDEX_MAP.columns[ col_index ]
}

#[inline]
pub fn box_at(box_index: usize) -> [usize; 9] {
    INDEX_MAP.boxes[ box_index ]
}

#[inline]
pub fn _row_containing(cell_index: usize) -> [usize; 9] {
    INDEX_MAP.rows[ row_index(cell_index) ]
}

#[inline]
pub fn _col_containing(cell_index: usize) -> [usize; 9] {
    INDEX_MAP.columns[ col_index(cell_index) ]
}

#[inline]
pub fn _box_containing(cell_index: usize) -> [usize; 9] {
    INDEX_MAP.boxes[ box_index(cell_index) ]
}

#[inline]
pub fn rcb_containing(cell_index: usize) -> &'static HashSet<usize> {
    &INDEX_MAP.rcb[cell_index]
}
