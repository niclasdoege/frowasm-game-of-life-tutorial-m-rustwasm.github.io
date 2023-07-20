mod utils;

//use std::os::fd::AsRawFd;

use js_sys::Math::log;
use wasm_bindgen::prelude::*;

extern crate js_sys;

extern crate web_sys;

extern crate fixedbitset;
use fixedbitset::FixedBitSet;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    //cells: Vec<Cell>,
}


/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.toggle(idx);
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        self.cells.toggle(self.get_index(row, column));
        self.cells.toggle(self.get_index(row, column+1));
        self.cells.toggle(self.get_index(row, column+2));
        self.cells.toggle(self.get_index(row-1, column+2));
        self.cells.toggle(self.get_index(row-2, column+1));
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        let pulsar05712  = [false,false,true,true,true,false,false,false,true,true,true,false,false];
        let pulsar1116   = [false,false,false,false,false,false,false,false,false,false,false,false,false];
        let pulsar2348910= [true,false,false,false,false,true,false,true,false,false,false,false,true];
        for horizontal in 0..13{     
            match horizontal{
                0|5|7|12=>    {for (i,x) in pulsar05712.iter().enumerate(){self.cells.set(self.get_index(row+horizontal, column+i as u32),x.to_owned() );}}
                1|11|6=>      {for (i,x) in pulsar1116.iter().enumerate(){self.cells.set(self.get_index(row+horizontal, column+i as u32),x.to_owned());}}
                2|3|4|8|9|10=>{for (i,x) in pulsar2348910.iter().enumerate(){self.cells.set(self.get_index(row+horizontal, column+i as u32),x.to_owned());}}
                _ =>{}
            }
            
        }
    }

    pub fn set_width(&mut self, width: u32){
        self.width=width;
        self.cells = FixedBitSet::with_capacity((width  * self.height) as usize);
        //self.cells = (0..width * self.height).map(|_| 0).collect();
        println!("{:?}",self.cells)
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = FixedBitSet::with_capacity((self.width  * height) as usize);
        //self.cells = (0..self.width * height).map(|_| 0).collect();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clear(&mut self){
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            self.cells.set(i, false);
        }
    }

    pub fn reset(&mut self){
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            self.cells.set(i, js_sys::Math::random() < 0.5);
        }
    }
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);


        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5);
        }
    
        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                log!(
                    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    cell,
                    live_neighbors
                );
               
                next.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise
                });

            }
        }
        //let difference = self.cells.difference(&next);
        let mut differentbits:Vec<(u32, u32)> = vec![];
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                if self.cells[idx] != next[idx] {
                    differentbits.push((row,col));
                    log!(
                        "cell[{}, {}] is initially {:?} and has {:?} neighbors. it becomes {:?}",
                        row,
                        col,
                        self.cells[idx],
                        self.live_neighbor_count(row, col),
                        next[idx]
                    );
                }
                //let difference_at = difference[idx];
            }
        }

        
            
        self.cells = next;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        ((row%self.height) * self.width + (column%self.width)) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

//non wasm exposed for testing
impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.insert(idx)
        }
    }

}
