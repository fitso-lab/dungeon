// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // コンソールウィンドウ無しで起動時

use eframe::egui::{self};
use eframe::{
    epaint::{Color32, Pos2, Rect, Rounding, Shape, Stroke, Vec2},
    Frame,
};

use rand::prelude::*;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(650.0, 650.0)),
        follow_system_theme: false,

        ..Default::default()
    };

    // MyApp は、ヒープに保持するため Box を使用する
    eframe::run_native(
        "My egui App",
        options,
        Box::new(move |_cc| {
            let mut t = Box::<MyApp>::default();
            t.rng = rand::thread_rng();

            t.width = 129;
            t.height = 111;

            t.cell_size = 5;

            t.make_maze();
            t.route_search(2, 2, 1);
            return t;
        }),
    )
}

#[derive(Clone, PartialEq)]
enum Cell {
    Wall = 1,
    Way = 0,
    Route = 2,
    Goal = 5,
}

#[derive(Default)]
struct Rand {
    /// 乱数の種
    rng: ThreadRng,
}

impl Rand {
    /// 指定した半径内の座標をランダム作成。乱数は、-1.0から1.0の疑似正規分布風
    #[allow(dead_code)]
    fn normal(&mut self, n: u32) -> f32 {
        let mut r: f32 = 0.0;
        for _ in 0..n {
            r = r + self.rng.gen_range(-1.0..=1.0);
        }
        return r / (n as f32);
    }
}

#[derive(Default)]
struct MyApp {
    /// 乱数の種
    rng: ThreadRng,

    /// 迷路の大きさ。処理を簡易にするために、一番外側を道、２番めを壁にしておく。
    /// 上の条件から、幅、高さとも奇数
    /// 最小の大きさは、５。ただし、道ひとマスだけなので迷路とは言えない。(^^;
    width: usize,
    height: usize,

    /// 迷路のマス
    maze: Vec<Vec<Cell>>,

    /// 表示する際のマスの大きさ
    cell_size: usize,
}

impl MyApp {
    #[allow(dead_code)]
    fn new() -> Self {
        let mut data = Self {
            rng: rand::thread_rng(),

            width: 129,
            height: 111,

            cell_size: 5,

            ..Self::default()
        };

        data.make_maze();
        return data;
    }

    fn re_new(&mut self) {
        // 壁で埋めた迷路領域を作成

        self.maze = vec![vec![Cell::Wall; self.height]; self.width];

        println!("({}, {})", self.width, self.height);

        // 外側を道で埋める（上下辺）
        for x in 0..self.width {
            self.maze[x][0] = Cell::Way;
            self.maze[x][self.height - 1] = Cell::Way;
        }

        // 外側を道で埋める（左右辺）
        for y in 0..self.height {
            self.maze[0][y] = Cell::Way;
            self.maze[self.width - 1][y] = Cell::Way;
        }

        self.make_maze();

        self.route_search(2, 2, 1);
    }

    /// 迷路内の道を作るための起点をすべて列挙する。
    fn list_xy(&self) -> Vec<(usize, usize)> {
        let mut xy = Vec::new();
        for x in (2..self.width - 2).step_by(2) {
            for y in (2..self.height - 2).step_by(2) {
                if self.maze[x][y] == Cell::Way {
                    for (dx, dy) in vec![(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if self.maze[(x as i32 + 2 * dx) as usize][(y as i32 + 2 * dy) as usize]
                            == Cell::Wall
                        {
                            xy.push((x, y));
                        }
                    }
                }
            }
        }
        // println!("起点の数({})", xy.len());
        return xy;
    }

    fn make_way(&mut self, mut x: usize, mut y: usize) {
        let d = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut old_v = None;
        let mut v;

        let l = if self.width > self.height {
            self.width
        } else {
            self.height
        };
        let l = self.rng.gen_range(1..(l / 3) + 1) + 1;
        for _ in 0..l {
            // 道を作成可能な方向を列挙する
            let mut ways = Vec::<(i32, i32)>::new();
            for v in &d {
                if self.maze[(x as i32 + v.0 * 2) as usize][(y as i32 + v.1 * 2) as usize]
                    == Cell::Wall
                {
                    ways.push((v.0, v.1));
                }
            }
            if ways.len() == 0 {
                // 道が作れなくなったら終了
                return;
            }
            let i = self.rng.gen_range(0..ways.len());
            if old_v.is_some() {
                let o_v: (i32, i32) = old_v.unwrap();
                if self.rng.gen_range(0.0..=1.0) < 0.4 {
                    v = ways[i];
                } else {
                    if self.maze[(x as i32 + o_v.0 * 2) as usize][(y as i32 + o_v.1 * 2) as usize]
                        == Cell::Wall
                    {
                        v = o_v;
                    } else {
                        v = ways[i];
                    }
                }
            } else {
                v = ways[i];
            }

            old_v = Some(v);

            for _ in 0..2 {
                x = (x as i32 + v.0) as usize;
                y = (y as i32 + v.1) as usize;
                self.maze[(x as i32) as usize][(y as i32) as usize] = Cell::Way;
            }
        }
    }

    fn make_maze(&mut self) {
        // 壁で埋めた迷路領域を作成
        self.maze = vec![vec![Cell::Wall; self.height]; self.width];

        println!("({}, {})", self.width, self.height);

        // 外側を道で埋める（上下辺）
        for x in 0..self.width {
            self.maze[x][0] = Cell::Way;
            self.maze[x][self.height - 1] = Cell::Way;
        }

        // 外側を道で埋める（左右辺）
        for y in 0..self.height {
            self.maze[0][y] = Cell::Way;
            self.maze[self.width - 1][y] = Cell::Way;
        }

        // 適当な座標に道を開ける
        let x = self.rng.gen_range(1..(self.width - 2) / 2) * 2;
        let y = self.rng.gen_range(1..(self.height - 2) / 2) * 2;

        self.maze[x][y] = Cell::Way;

        // 道を伸ばせるだけ伸ばす
        self.make_way(x, y);

        loop {
            // 迷路から道を探して次の道をの明日候補としてリストアップする
            let mut xy = self.list_xy();

            // 起点リストからアットランダムに起点を選ぶ。選んだ起点はリストから削除する
            if xy.len() == 0 {
                println!("迷路完成！");
                break;
            }
            let i = self.rng.gen_range(0..xy.len());
            let (x, y) = xy[i];
            xy.remove(i);

            self.maze[x][y] = Cell::Way;
            // 道を伸ばせるだけ伸ばす
            self.make_way(x, y);
        }

        let x = self.width - 3;
        let y = self.height - 3;
        self.maze[x][y] = Cell::Goal;

        // println!("Goal ({}, {})", x, y);
    }

    fn route_search(&mut self, x: usize, y: usize, depth: usize) -> bool {
        let d = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];

        if self.maze[x][y] == Cell::Goal {
            println!("Goal ({}, {}):{}", x, y, depth);
            return true;
        }

        self.maze[x][y] = Cell::Route;
        for (vx, vy) in &d {
            let wx = (x as i32 + vx) as usize;
            let wy = (y as i32 + vy) as usize;
            if self.maze[wx][wy] == Cell::Way || self.maze[wx][wy] == Cell::Goal {
                // println!("Route({}, {}):{}", x, y, depth);
                if self.route_search(
                    (x as i32 + vx) as usize,
                    (y as i32 + vy) as usize,
                    depth + 1,
                ) {
                    return true;
                }
            }
        }

        self.maze[x][y] = Cell::Way;

        return false;
    }

    fn rect_from_min_size(&self, x: usize, y: usize) -> Rect {
        let rect = Rect::from_min_size(
            Pos2 {
                x: (x * self.cell_size) as f32,
                y: (y * self.cell_size) as f32,
            },
            Vec2 {
                x: self.cell_size as f32,
                y: self.cell_size as f32,
            },
        );

        return rect;
    }

    fn rect_fill(&self, shapes: &mut Vec<Shape>, rect: Rect, c: &Cell) {
        let col;
        let stroke;

        match c {
            // 道
            Cell::Way => {
                col = Color32::LIGHT_GREEN;
                stroke = Stroke::new(1.0, Color32::GREEN);
            }
            // 壁
            Cell::Wall => {
                col = Color32::BROWN;
                stroke = Stroke::new(1.0, Color32::BROWN);
            }
            Cell::Route => {
                col = Color32::GOLD;
                stroke = Stroke::new(1.0, Color32::GOLD);
            }
            Cell::Goal => {
                col = Color32::RED;
                stroke = Stroke::new(1.0, Color32::LIGHT_RED);
            } // _ => {}
        }

        shapes.push(Shape::rect_filled(rect, Rounding::none(), col));
        shapes.push(Shape::rect_stroke(rect, Rounding::none(), stroke));
    }

    fn draw_maze(&self) -> Vec<Shape> {
        let mut shapes = Vec::new();
        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                let rect = self.rect_from_min_size(x, y);

                self.rect_fill(&mut shapes, rect, &self.maze[x][y]);
            }
        }

        return shapes;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // let stroke = Stroke::new(1.0, Color32::LIGHT_GRAY);
            // let stroke2 = Stroke::new(1 as f32, Color32::LIGHT_GREEN);

            ui.heading("My egui Application");
            if ui.add(egui::Button::new("Re-Start")).clicked() {
                self.re_new();
            }
            let mut w = "".to_string();
            let response = ui.add(egui::TextEdit::singleline(&mut w));
            if response.changed() {
                if let Ok(_w) = w.parse::<u32>() {}
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.re_new();
            }
            let shapes = self.draw_maze();
            ui.painter().extend(shapes);
        });

        ctx.request_repaint();
    }
}
