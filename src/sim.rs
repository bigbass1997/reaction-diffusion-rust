use minifb::clamp;
use crate::util::InfCell;

#[derive(Default, Debug)]
pub struct Cell {
    neighbors: Vec<&'static Cell>,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<InfCell<Cell>>,
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = vec![];
        for i in 0..(width * height) {
            let mut cell = Cell::default();
            cell.a = 1.0;
            cell.b = 0.0;
            cells.push(InfCell::new(cell));
        }

        {
            let width = width as isize;
            let height = height as isize;
            for i in 0..cells.len() {
                
                let cell = cells[i].get_mut();
                let cx = i as isize % width;
                let cy = i as isize / width;
                
                for y in -1..=1 {
                    for x in -1..=1 {
                        let mut nx = cx + x;
                        let mut ny = cy + y;
                        
                        if nx < 0 {
                            nx = width + nx;
                        } else if nx >= width {
                            nx %= width;
                        }
                        
                        if ny < 0 {
                            ny = height + ny;
                        } else if ny >= height {
                            ny %= height;
                        }
                        
                        let i = ((ny * width) + nx) as usize;
                        
                        cell.neighbors.push(cells[i].get());
                    }
                }
            }
        }
        
        Self {
            width,
            height,
            cells,
        }
    }
}


#[derive(Debug)]
pub struct Simulation {
    pub   da: f64,
    pub   db: f64,
    pub    f: f64,
    pub    k: f64,
    pub  adj: f64,
    pub diag: f64,
    
    cur_grid: Grid,
    nex_grid: Grid,
    pub framebuffer: Box<[u32]>,
}
impl Simulation {
    pub fn new(da: f64, db: f64, f: f64, k: f64, adj: f64, diag: f64, width: usize, height: usize) -> Self { Self {
        da,
        db,
        f,
        k,
        adj,
        diag,
        
        cur_grid: Grid::new(width, height),
        nex_grid: Grid::new(width, height),
        framebuffer: vec![0u32; width * height].into_boxed_slice(),
    }}
    
    pub fn seed(&mut self) {
        let mut rng = oorandom::Rand64::new(8972134);
        let width = self.cur_grid.width;
        let height = self.cur_grid.height;
        
        let lo = (width * 4) as u64;
        let hi = ((width * height) as u64) - lo;
        
        // random set of '+' seeds
        for _ in 0..128 {
            let p = rng.rand_range(lo..hi) as usize;
            self.cur_grid.cells[p-1].get_mut().b = 1.0;
            self.cur_grid.cells[p].get_mut().b = 1.0;
            self.cur_grid.cells[p+1].get_mut().b = 1.0;
            self.cur_grid.cells[p-width].get_mut().b = 1.0;
            self.cur_grid.cells[p+width].get_mut().b = 1.0;
        }
    }
    
    pub fn generation(&mut self) {
        for i in 0..self.cur_grid.cells.len() {
            let c = self.cur_grid.cells[i].get();
            let abb = c.a * c.b * c.b;
            let (lapa, lapb) = self.laplacian(c);
            
            let n = self.nex_grid.cells[i].get_mut();
            n.a = clamp(0.0, c.a + (self.da * lapa) - abb + (self.f * (1.0 - c.a)), 1.0);
            n.b = clamp(0.0, c.b + (self.db * lapb) + abb - ((self.k + self.f) * c.b), 1.0);
            
            let val = clamp(0.1f64, (n.a + n.b) * (n.a - n.b) * 2f64, 1.0);
            self.framebuffer[i] = from_f64_rgb(val, val, val);
        }
        
        self.swap();
    }
    
    fn laplacian(&self, cell: &Cell) -> (f64, f64) {
        let u = &cell.neighbors[1];
        let d = &cell.neighbors[7];
        let l = &cell.neighbors[3];
        let r = &cell.neighbors[5];
        
        let lu = &cell.neighbors[0];
        let ru = &cell.neighbors[2];
        let ld = &cell.neighbors[6];
        let rd = &cell.neighbors[8];
        
        (
            -cell.a + ((u.a + d.a + l.a + r.a) * self.adj) + ((lu.a + ru.a + ld.a + rd.a) * self.diag),
            -cell.b + ((u.b + d.b + l.b + r.b) * self.adj) + ((lu.b + ru.b + ld.b + rd.b) * self.diag)
        )
    }
    
    fn swap(&mut self) {
        std::mem::swap(&mut self.cur_grid, &mut self.nex_grid);
    }
}

fn from_f64_rgb(r: f64, g: f64, b: f64) -> u32 {
    (((r * 255.0) as u32) << 16) | (((g * 255.0) as u32) << 8) | ((b * 255.0) as u32)
}