//! Hive board PNG export mirroring hive/visualize.py projection and styling, but drawn as
//! **flat‑top** hexes (horizontal top/bottom edges) instead of matplotlib’s default pointy‑top.

use crate::{Board, Color as HiveColor, PieceType, Position};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::path::PathBuf;

const HEX_RADIUS: f64 = 1.0;
/// First‑vertex polar angle (`RegularPolygon.orientation` analogue). π/3 is π/6 left of π/2,
/// yielding flat‑top hexes whose top and bottom sides are horizontal.
const HEX_ORIENTATION: f64 = std::f64::consts::FRAC_PI_3;
const RAD_60: f64 = std::f64::consts::PI / 3.0;

#[inline]
fn position_to_plane_xy(pos: Position) -> (f64, f64) {
    let r = f64::from(pos.r);
    let s = f64::from(pos.s);
    let q = f64::from(pos.q);
    let px = q - RAD_60.cos() * (r + s);
    let py = RAD_60.sin() * (s - r);
    (px, py)
}

fn grid_positions_python() -> Vec<Position> {
    let mut v = Vec::new();
    for q in -10_i8..=10 {
        for s in -10_i8..=10 {
            let r = -q - s;
            if (-10_i8..=10).contains(&r) {
                v.push(Position::new(q, s, r).expect("cube constraint holds"));
            }
        }
    }
    v.sort_by_key(|p| (p.q, p.s, p.r));
    v
}

fn hex_polygon_vertices(px: f64, py: f64) -> [(f64, f64); 6] {
    let mut verts = [(0., 0.); 6];
    for k in 0_usize..6 {
        let ang = HEX_ORIENTATION + k as f64 * std::f64::consts::FRAC_PI_3;
        verts[k] = (px + HEX_RADIUS * ang.cos(), py + HEX_RADIUS * ang.sin());
    }
    verts
}

fn hex_outline_path(verts: &[(f64, f64); 6]) -> Vec<(f64, f64)> {
    let mut pts: Vec<(f64, f64)> = verts.to_vec();
    pts.push(verts[0]);
    pts
}

fn cube_axis_label_offsets(hex_r: f64) -> ((f64, f64), (f64, f64), (f64, f64)) {
    let inset = hex_r * 0.58;
    let ks = [0_usize, 2, 4];
    let mut out = [(0., 0.); 3];
    for (i, k) in ks.iter().enumerate() {
        let ang = HEX_ORIENTATION + *k as f64 * std::f64::consts::FRAC_PI_3;
        out[i] = (inset * ang.cos(), inset * ang.sin());
    }
    (out[0], out[1], out[2])
}

#[inline]
fn piece_face_rgb(pt: PieceType) -> RGBColor {
    match pt {
        PieceType::Queen => RGBColor(0xf1, 0xc4, 0x0f),
        PieceType::Grasshopper => RGBColor(0x27, 0xae, 0x60),
        PieceType::SoldierAnt => RGBColor(0x29, 0x80, 0xb9),
        PieceType::Spider => RGBColor(0x8b, 0x45, 0x13),
        PieceType::Beetle => RGBColor(0x8e, 0x44, 0xad),
    }
}

#[inline]
fn piece_glyph(pt: PieceType) -> &'static str {
    match pt {
        PieceType::Queen => "Q",
        PieceType::Grasshopper => "G",
        PieceType::SoldierAnt => "A",
        PieceType::Spider => "S",
        PieceType::Beetle => "B",
    }
}

#[inline]
fn cube_label_rgb(board: &Board, pos: &Position) -> RGBColor {
    match board.get_top_piece(pos) {
        None => RGBColor(0x2c, 0x2c, 0x2c),
        Some(p) if p.color == HiveColor::Black => RGBColor(0xe8, 0xe8, 0xe8),
        Some(_) => RGBColor(0x1e, 0x1e, 0x1e),
    }
}

fn legend_square(cx: f64, cy: f64, half: f64) -> Vec<(f64, f64)> {
    vec![
        (cx - half, cy - half),
        (cx + half, cy - half),
        (cx + half, cy + half),
        (cx - half, cy + half),
        (cx - half, cy - half),
    ]
}

/// Save PNG at `../hive_view_rust.png` relative to [`rust_impl/`](crate) (parallel to Python’s repo-root `hive_view.png`).
pub fn save_hive_png(board: &Board, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../hive_view_rust.png");

    let grid = grid_positions_python();
    let projected: Vec<(f64, f64)> = grid.iter().copied().map(position_to_plane_xy).collect();
    let xmin = projected.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let xmax = projected
        .iter()
        .map(|p| p.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let ymin = projected.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let ymax = projected
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = HEX_RADIUS + 0.35;
    let x_rng = xmin - pad..xmax + pad;
    let y_rng = ymin - pad..ymax + pad;

    let (dq_label, ds_label, dr_label) = cube_axis_label_offsets(HEX_RADIUS);

    let root = BitMapBackend::new(&out_path, (1200, 1050)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22).into_font())
        .margin(10)
        .x_label_area_size(42)
        .y_label_area_size(54)
        .build_cartesian_2d(x_rng.clone(), y_rng.clone())?;

    chart
        .configure_mesh()
        .x_desc("Plane x")
        .y_desc("Plane y")
        .light_line_style(RGBColor(200, 200, 200).mix(0.45).stroke_width(1))
        .x_label_formatter(&|v| format!("{v:.1}"))
        .y_label_formatter(&|v| format!("{v:.1}"))
        .draw()?;

    let pa = chart.plotting_area();

    let empty_hex = RGBColor(208, 208, 208).mix(0.55);
    let empty_edge = RGBColor(136, 136, 136).stroke_width(1);
    let white_hex = RGBColor(248, 244, 232).mix(0.95);
    let white_edge = RGBColor(51, 51, 51).stroke_width(1);
    let black_hex = RGBColor(26, 26, 26).mix(0.95);
    let black_edge = RGBColor(102, 102, 102).stroke_width(1);

    #[derive(Clone, Copy)]
    enum CellTone {
        Empty,
        Light,
        Dark,
    }

    for pos in &grid {
        let (px, py) = position_to_plane_xy(*pos);
        let verts = hex_polygon_vertices(px, py);
        let tone = match board.get_top_piece(pos) {
            None => CellTone::Empty,
            Some(p) if p.color == HiveColor::White => CellTone::Light,
            Some(_) => CellTone::Dark,
        };
        let (fillc, strokec) = match tone {
            CellTone::Empty => (empty_hex, empty_edge.clone()),
            CellTone::Light => (white_hex, white_edge.clone()),
            CellTone::Dark => (black_hex, black_edge.clone()),
        };
        pa.draw(&Polygon::new(verts.to_vec(), fillc))?;
        pa.draw(&PathElement::new(hex_outline_path(&verts), strokec))?;
    }

    let center_radius = HEX_RADIUS * 0.42_f64;

    let glyph_anchor = Pos::new(HPos::Center, VPos::Center);

    for pos in &grid {
        if let Some(piece) = board.get_top_piece(pos) {
            let (px, py) = position_to_plane_xy(*pos);
            // Plain RGBAColor → ShapeStyle defaults to unfilled (stroke-only); use `.filled()`
            // so each insect shows its palette color instead of a nearly invisible ring.
            let fill = piece_face_rgb(piece.piece_type).mix(0.97).filled();
            pa.draw(&Circle::new((px, py), center_radius, fill))?;
            let rim = match piece.color {
                HiveColor::White => RGBColor(34, 34, 34).stroke_width(2),
                HiveColor::Black => RGBColor(204, 204, 204).stroke_width(2),
            };
            pa.draw(&Circle::new((px, py), center_radius, rim))?;

            let glyph_color = match piece.color {
                HiveColor::White => RGBColor(22, 22, 22),
                HiveColor::Black => RGBColor(240, 240, 240),
            };
            let glyph_style = ("sans-serif", 14_u32)
                .with_color(&glyph_color)
                .with_anchor::<RGBColor>(glyph_anchor)
                .into_text_style(pa);
            pa.draw(&Text::new(
                piece_glyph(piece.piece_type),
                (px, py),
                glyph_style,
            ))?;
        }
    }

    let cube_anchor = Pos::new(HPos::Center, VPos::Center);

    for pos in &grid {
        let (px, py) = position_to_plane_xy(*pos);
        let rgb = cube_label_rgb(board, pos);

        let label_q = ("sans-serif", 11_u32)
            .with_color(&rgb)
            .with_anchor::<RGBColor>(cube_anchor)
            .into_text_style(pa);

        pa.draw(&Text::new(
            format!("{}", pos.q),
            (px + dq_label.0, py + dq_label.1),
            label_q,
        ))?;

        let label_r = ("sans-serif", 11_u32)
            .with_color(&rgb)
            .with_anchor::<RGBColor>(cube_anchor)
            .into_text_style(pa);

        pa.draw(&Text::new(
            format!("{}", pos.r),
            (px + dr_label.0, py + dr_label.1),
            label_r,
        ))?;

        let label_s = ("sans-serif", 11_u32)
            .with_color(&rgb)
            .with_anchor::<RGBColor>(cube_anchor)
            .into_text_style(pa);

        pa.draw(&Text::new(
            format!("{}", pos.s),
            (px + ds_label.0, py + ds_label.1),
            label_s,
        ))?;
    }

    let lx = x_rng.end - 6.85_f64;
    let mut ly = y_rng.end - pad * 1.05;

    let legend_rows: &[(&str, RGBColor)] = &[
        ("Empty block", RGBColor(208, 208, 208)),
        ("White piece", RGBColor(248, 244, 232)),
        ("Black piece", RGBColor(26, 26, 26)),
        ("Queen", piece_face_rgb(PieceType::Queen)),
        ("Grasshopper", piece_face_rgb(PieceType::Grasshopper)),
        ("Ant", piece_face_rgb(PieceType::SoldierAnt)),
        ("Spider", piece_face_rgb(PieceType::Spider)),
        ("Beetle", piece_face_rgb(PieceType::Beetle)),
    ];

    let entry_text_anchor = Pos::new(HPos::Left, VPos::Center);

    let entry_font = ("sans-serif", 13_u32)
        .with_color(&BLACK)
        .with_anchor::<RGBColor>(entry_text_anchor)
        .into_text_style(pa);

    let square_half = HEX_RADIUS * 0.125;
    let row_step = HEX_RADIUS * 0.52;

    for (label, color) in legend_rows {
        ly -= row_step * 1.06;
        let cx = lx + HEX_RADIUS * 0.35;
        let cy = ly;
        let outline = RGBColor(90, 90, 90).stroke_width(1);
        pa.draw(&Polygon::new(
            legend_square(cx, cy, square_half),
            color.mix(0.94),
        ))?;
        pa.draw(&PathElement::new(
            legend_square(cx, cy, square_half),
            outline.clone(),
        ))?;

        pa.draw(&Text::new(
            (*label).to_string(),
            (cx + HEX_RADIUS * 0.72, cy),
            entry_font.clone(),
        ))?;
    }

    root.present()?;

    println!("Hive plot saved to {}", out_path.display());
    Ok(())
}
