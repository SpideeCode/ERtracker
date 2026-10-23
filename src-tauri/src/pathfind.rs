//! Route finding over walkable ground samples (walkable.bin): A* on an implicit
//! graph where nearby samples are linked when the slope can be walked (or
//! dropped down), and dungeons connect to the open world only at their entrance.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::path::Path;

/// Neighbour radius inside dungeons and in the open world (where props are
/// sparse), metres. The grid cell is the larger one.
const RADIUS: f32 = 9.0;
const OPEN_RADIUS: f32 = 35.0;
const CELL: f32 = OPEN_RADIUS;
/// Highest climb per metre walked, and extra height that can be dropped.
const MAX_SLOPE: f32 = 0.9;
/// Long open-world links: gentler slope, and ground required around the midpoint
/// (otherwise the link would jump over a ravine or a cliff edge).
const LONG_SLOPE: f32 = 0.6;
const SUPPORT_RADIUS: f32 = 10.0;
const SUPPORT_HEIGHT: f32 = 6.0;
const MAX_DROP: f32 = 10.0;
/// Snapping a point (grace, boss...) to the graph.
const SNAP: f32 = 40.0;
const MAX_EXPANSIONS: usize = 1_500_000;

#[derive(Clone, Copy)]
struct Node {
    x: f32,
    y: f32,
    z: f32,
    area: u8,
    gx: u8,
}

impl Node {
    /// Open world, legacy dungeons and the underground connect physically;
    /// catacombs, caves, tunnels... only through their entrance.
    fn small_dungeon(&self) -> bool {
        !matches!(self.area, 60 | 61 | 12) && self.area >= 30
    }
    fn open(&self) -> bool {
        matches!(self.area, 60 | 61 | 12)
    }
    fn same_block(&self, o: &Node) -> bool {
        self.area == o.area && (self.gx == o.gx || matches!(self.area, 60 | 61))
    }
}

struct MapGraph {
    nodes: Vec<Node>,
    grid: HashMap<(i32, i32), Vec<u32>>,
    /// Extra links between a dungeon's entrance area and its interior.
    portals: HashMap<u32, Vec<u32>>,
}

pub struct Graph {
    maps: [MapGraph; 4],
}

#[derive(Deserialize)]
struct Place {
    icon: u16,
    block: String,
    map: String,
    x: f32,
    y: f32,
    height: f32,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize)]
pub struct PathResult {
    pub found: bool,
    /// Simplified polyline, map pixels and height.
    pub points: Vec<[f32; 3]>,
    pub length: f32,
    pub ascent: f32,
    pub descent: f32,
}

fn cell(x: f32, y: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (y / CELL).floor() as i32)
}

fn map_index(map: &str) -> Option<usize> {
    ["M00", "M01", "M10", "M11"].iter().position(|m| *m == map)
}

impl MapGraph {
    fn nearby(&self, x: f32, y: f32) -> impl Iterator<Item = u32> + '_ {
        let (cx, cy) = cell(x, y);
        (-1..=1).flat_map(move |dx| (-1..=1).map(move |dy| (cx + dx, cy + dy))).flat_map(|c| self.grid.get(&c).into_iter().flatten().copied())
    }

    fn nearest(&self, p: Point, filter: impl Fn(&Node) -> bool) -> Option<u32> {
        let (cx, cy) = cell(p.x, p.y);
        let reach = (SNAP / CELL).ceil() as i32;
        let mut best: Option<(f32, u32)> = None;
        for dx in -reach..=reach {
            for dy in -reach..=reach {
                for &i in self.grid.get(&(cx + dx, cy + dy)).into_iter().flatten() {
                    let n = &self.nodes[i as usize];
                    if !filter(n) {
                        continue;
                    }
                    let h = ((n.x - p.x).powi(2) + (n.y - p.y).powi(2)).sqrt();
                    if h > SNAP {
                        continue;
                    }
                    let score = h + 2.0 * (n.z - p.z).abs();
                    if best.is_none_or(|(s, _)| score < s) {
                        best = Some((score, i));
                    }
                }
            }
        }
        best.map(|(_, i)| i)
    }

    /// Some ground sample near a point at about that height.
    fn supported(&self, x: f32, y: f32, z: f32) -> bool {
        self.nearby(x, y).any(|i| {
            let n = &self.nodes[i as usize];
            (n.x - x).powi(2) + (n.y - y).powi(2) < SUPPORT_RADIUS * SUPPORT_RADIUS && (n.z - z).abs() < SUPPORT_HEIGHT
        })
    }

    /// Walkable step from a to b, and its cost.
    fn step(&self, a: &Node, b: &Node) -> Option<f32> {
        let h = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        if h > if a.open() && b.open() { OPEN_RADIUS } else { RADIUS } {
            return None;
        }
        let dz = b.z - a.z;
        let slope = if h > RADIUS { LONG_SLOPE } else { MAX_SLOPE };
        if dz > 1.0 + slope * h || -dz > MAX_DROP + slope * h {
            return None;
        }
        if h > RADIUS && !self.supported((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0) {
            return None;
        }
        if (a.small_dungeon() || b.small_dungeon()) && !a.same_block(b) {
            return None;
        }
        Some((h * h + dz * dz).sqrt() + 0.5 * dz.max(0.0))
    }
}

#[derive(PartialEq)]
struct Open(f32, u32);
impl Eq for Open {}
impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Open {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

impl Graph {
    pub fn load(dir: &Path) -> Option<Self> {
        let data = std::fs::read(dir.join("walkable.bin")).ok()?;
        if data.get(0..4) != Some(b"ERWK") {
            return None;
        }
        let count = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;
        let mut maps: [MapGraph; 4] =
            std::array::from_fn(|_| MapGraph { nodes: Vec::new(), grid: HashMap::new(), portals: HashMap::new() });
        let f = |o: usize| f32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        for i in 0..count {
            let o = 12 + i * 16;
            let Some(rec) = data.get(o..o + 16) else { break };
            let m = rec[0] as usize;
            if m > 3 {
                continue;
            }
            let node = Node { area: rec[1], gx: rec[2], x: f(o + 4), y: f(o + 8), z: f(o + 12) };
            let g = &mut maps[m];
            g.grid.entry(cell(node.x, node.y)).or_default().push(g.nodes.len() as u32);
            g.nodes.push(node);
        }

        // Dungeon entrances: link the open ground at the entrance icon with the
        // dungeon interior nearest to it, both ways.
        let places: Vec<Place> =
            std::fs::read(dir.join("places.json")).ok().and_then(|d| serde_json::from_slice(&d).ok()).unwrap_or_default();
        for p in places {
            let Some(m) = map_index(&p.map) else { continue };
            let g = &mut maps[m];
            let parts: Vec<u8> = p.block.trim_start_matches('m').split('_').filter_map(|s| s.parse().ok()).collect();
            let [area, gx, ..] = parts.as_slice() else { continue };
            let at = Point { x: p.x, y: p.y, z: p.height };
            let probe = Node { area: *area, gx: *gx, x: 0.0, y: 0.0, z: 0.0 };
            if !probe.small_dungeon() {
                // Entrance icon placed in the open world: link it to the nearest dungeon block below.
                if !matches!(p.icon, 4 | 13 | 14 | 16 | 230 | 231 | 232 | 233 | 234) {
                    continue;
                }
            }
            let outside = g.nearest(at, |n| !n.small_dungeon());
            let inside = if probe.small_dungeon() {
                g.nearest(Point { z: at.z, ..at }, |n| n.area == *area && n.gx == *gx).or_else(|| {
                    // The interior often starts well below the icon: widen the search vertically.
                    g.nearby(at.x, at.y).filter(|&i| g.nodes[i as usize].area == *area && g.nodes[i as usize].gx == *gx).next()
                })
            } else {
                g.nearby(at.x, at.y).find(|&i| g.nodes[i as usize].small_dungeon())
            };
            if let (Some(a), Some(b)) = (outside, inside) {
                g.portals.entry(a).or_default().push(b);
                g.portals.entry(b).or_default().push(a);
            }
        }
        Some(Self { maps })
    }

    pub fn find(&self, map: &str, from: Point, to: Point) -> PathResult {
        let none = PathResult { found: false, points: vec![[from.x, from.y, from.z], [to.x, to.y, to.z]], length: 0.0, ascent: 0.0, descent: 0.0 };
        let Some(g) = map_index(map).map(|m| &self.maps[m]) else { return none };
        let (Some(start), Some(goal)) = (g.nearest(from, |_| true), g.nearest(to, |_| true)) else { return none };
        let goal_node = g.nodes[goal as usize];
        let h = |i: u32| {
            let n = &g.nodes[i as usize];
            ((n.x - goal_node.x).powi(2) + (n.y - goal_node.y).powi(2)).sqrt()
        };

        let mut cost: HashMap<u32, f32> = HashMap::from([(start, 0.0)]);
        let mut came: HashMap<u32, u32> = HashMap::new();
        let mut open = BinaryHeap::from([Open(h(start), start)]);
        let mut expansions = 0;
        while let Some(Open(_, i)) = open.pop() {
            if i == goal {
                break;
            }
            expansions += 1;
            if expansions > MAX_EXPANSIONS {
                return none;
            }
            let a = g.nodes[i as usize];
            let base = cost[&i];
            let portals = g.portals.get(&i).into_iter().flatten().map(|&j| (j, 5.0));
            let steps = g.nearby(a.x, a.y).filter(|&j| j != i).filter_map(|j| g.step(&a, &g.nodes[j as usize]).map(|c| (j, c)));
            for (j, c) in steps.chain(portals) {
                let next = base + c;
                if cost.get(&j).is_none_or(|&old| next < old) {
                    cost.insert(j, next);
                    came.insert(j, i);
                    open.push(Open(next + h(j), j));
                }
            }
        }
        if !came.contains_key(&goal) && start != goal {
            return none;
        }

        let mut chain = vec![goal];
        while let Some(&prev) = came.get(chain.last().unwrap()) {
            chain.push(prev);
        }
        chain.reverse();
        let mut points: Vec<[f32; 3]> = std::iter::once([from.x, from.y, from.z])
            .chain(chain.iter().map(|&i| {
                let n = g.nodes[i as usize];
                [n.x, n.y, n.z]
            }))
            .chain(std::iter::once([to.x, to.y, to.z]))
            .collect();
        let (mut length, mut ascent, mut descent) = (0.0, 0.0, 0.0);
        for w in points.windows(2) {
            let dz = w[1][2] - w[0][2];
            length += ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
            if dz > 0.0 {
                ascent += dz;
            } else {
                descent -= dz;
            }
        }
        points = simplify(&points, 3.0);
        PathResult { found: true, points, length, ascent, descent }
    }
}

/// Douglas-Peucker on the horizontal plane, keeping every point where height changes a lot.
fn simplify(points: &[[f32; 3]], tolerance: f32) -> Vec<[f32; 3]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let (a, b) = (points[0], points[points.len() - 1]);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let (mut worst, mut index) = (0.0, 0);
    for (i, p) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let d = ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / len;
        let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (len * len)).clamp(0.0, 1.0);
        let dz = (p[2] - (a[2] + t * (b[2] - a[2]))).abs() / 3.0;
        if d.max(dz) > worst {
            worst = d.max(dz);
            index = i;
        }
    }
    if worst <= tolerance {
        return vec![a, b];
    }
    let mut left = simplify(&points[..=index], tolerance);
    left.pop();
    left.extend(simplify(&points[index..], tolerance));
    left
}
