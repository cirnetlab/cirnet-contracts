use super::{Catalogs, DesignResult, EPS, MM_TO_WORLD, list, numeric, text};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
    ops::{Add, Mul, Sub},
};

#[derive(Clone, Copy, Debug)]
pub(super) struct Point([f64; 3]);

impl Point {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }
    fn x(self) -> f64 {
        self.0[0]
    }
    fn y(self) -> f64 {
        self.0[1]
    }
    pub(super) fn z(self) -> f64 {
        self.0[2]
    }
    pub(super) fn is_finite(self) -> bool {
        self.0.into_iter().all(f64::is_finite)
    }
    fn dot(self, other: Self) -> f64 {
        self.0.into_iter().zip(other.0).map(|(a, b)| a * b).sum()
    }
    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }
    fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub(super) fn distance(self, other: Self) -> f64 {
        let delta = self - other;
        delta.x().hypot(delta.y()).hypot(delta.z())
    }
    fn unit(self) -> Self {
        self * (1.0 / self.norm())
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] + other.0[i]))
    }
}
impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] - other.0[i]))
    }
}
impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, factor: f64) -> Self {
        Self(self.0.map(|value| value * factor))
    }
}

type Matrix = [Point; 3];

pub(super) fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

pub(super) fn point(value: &Value) -> Point {
    Point(["x", "y", "z"].map(|axis| number(&value[axis])))
}

pub(super) fn coordinates(value: &Value) -> Point {
    Point([0, 1, 2].map(|axis| number(&value[axis])))
}

fn translation(module: &Value) -> Point {
    Point(["x", "y", "z"].map(|axis| module["position"].get(axis).map_or(0.0, number)))
}

fn rotation(value: &Point) -> Matrix {
    let [x, y, z] = value.0.map(f64::to_radians);
    let (b, a) = x.sin_cos();
    let (d, c) = y.sin_cos();
    let (f, e) = z.sin_cos();
    [
        Point::new(c * e + d * b * f, d * b * e - c * f, a * d),
        Point::new(a * f, a * e, -b),
        Point::new(c * b * f - d * e, c * b * e + d * f, a * c),
    ]
}

fn yaw_matrix(angle: f64) -> Matrix {
    rotation(&Point::new(0.0, angle, 0.0))
}

pub(super) fn yaw(angle: f64, value: Point) -> Point {
    transform(yaw_matrix(angle), value)
}

fn transpose(matrix: Matrix) -> Matrix {
    std::array::from_fn(|i| Point(std::array::from_fn(|j| matrix[j].0[i])))
}

fn transform(matrix: Matrix, value: Point) -> Point {
    Point(matrix.map(|axis| axis.dot(value)))
}

fn module_yaw(module: &Value) -> f64 {
    module.get("rotation").map_or(0.0, number)
}

fn integer_yaw(value: &Value) -> Option<f64> {
    let integer = numeric::lexical_integer(value)?;
    let remainder = i64::try_from(integer % 360_i64).expect("bounded angle remainder");
    Some((remainder as f64).rem_euclid(360.0))
}

pub(super) fn wrapped_yaw(value: &Value) -> f64 {
    integer_yaw(value).unwrap_or_else(|| number(value).rem_euclid(360.0))
}

fn combined_rigid_yaw(board: &Value, mount_yaw: &Value) -> f64 {
    let board_integer = board.get("rotation").map_or(Some(0.0), integer_yaw);
    if let (Some(board), Some(mount)) = (board_integer, integer_yaw(mount_yaw)) {
        return (board + mount).rem_euclid(360.0);
    }
    (module_yaw(board) + number(mount_yaw)).rem_euclid(360.0)
}

fn lead_radius_mm(package: &Value) -> f64 {
    let section = &package["shape"]["leadSection"];
    if section.is_null() {
        number(&package["leadDiameterMm"]) / 2.0
    } else if section["kind"] == "round" {
        number(&section["widthMm"]) / 2.0
    } else {
        number(&section["widthMm"])
            .max(section.get("shoulderWidthMm").map_or(0.0, number))
            .hypot(number(&section["thicknessMm"]))
            / 2.0
    }
}

struct LeadPath {
    points: Vec<Point>,
    samples: Vec<Point>,
    distances: Vec<f64>,
    length: f64,
    error: f64,
}

fn build_lead_path(
    raw: &[Point],
    minimum: f64,
    preferred: f64,
    straight: [f64; 2],
) -> DesignResult<LeadPath> {
    if !minimum.is_finite()
        || minimum <= 0.0
        || !preferred.is_finite()
        || preferred < minimum
        || straight
            .into_iter()
            .any(|length| !length.is_finite() || length < 0.0)
    {
        return Err("Invalid flexible lead bend radius or straight constraint".into());
    }
    let points = normalize_lead(raw)?;
    let last = points.len() - 1;
    let lengths: Vec<_> = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .collect();
    let directions: Vec<_> = points
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).unit())
        .collect();
    let mut tangents = vec![0.0; points.len()];
    let mut angles = vec![0.0; points.len()];
    for i in 1..last {
        let cosine = directions[i - 1].dot(directions[i]).clamp(-1.0, 1.0);
        if cosine < -1.0 + 1e-10 {
            return Err("Flexible lead reverses through itself".into());
        }
        angles[i] = cosine.acos();
        tangents[i] = (angles[i] / 2.0).tan();
    }
    let mut factors = vec![1.0_f64; points.len()];
    for (i, length) in lengths.into_iter().enumerate() {
        let reserved = (if i == 0 { straight[0] } else { 0.0 })
            + (if i == last - 1 { straight[1] } else { 0.0 });
        let available = length - reserved;
        let required = minimum * tangents[i] + minimum * tangents[i + 1];
        if required > available + 1e-9 {
            return Err(
                "Flexible lead cannot fit its minimum bend radius and endpoint straights".into(),
            );
        }
        let wanted = (preferred - minimum) * tangents[i] + (preferred - minimum) * tangents[i + 1];
        if wanted > 0.0 {
            let factor = ((available - required) / wanted).clamp(0.0, 1.0);
            factors[i] = factors[i].min(factor);
            factors[i + 1] = factors[i + 1].min(factor);
        }
    }
    let mut path = LeadPath {
        points: Vec::new(),
        samples: vec![points[0]],
        distances: vec![0.0],
        length: 0.0,
        error: (minimum / 512.0).min(1e-5),
    };
    let mut previous = points[0];
    for i in 1..last {
        if tangents[i] <= 1e-10 {
            continue;
        }
        let radius = minimum + (preferred - minimum) * factors[i];
        let before = directions[i - 1];
        let after = directions[i];
        let trim = radius * tangents[i];
        let start = points[i] - before * trim;
        let end = points[i] + after * trim;
        let axis = before.cross(after).unit();
        let center = start + axis.cross(before) * radius;
        push_line(&mut path, previous, start);
        let offset = start - center;
        let perpendicular = axis.cross(offset);
        let step = 2.0 * (1.0 - path.error / radius).clamp(-1.0, 1.0).acos();
        let count = (angles[i] / step).ceil().max(1.0) as usize;
        let length = radius * angles[i];
        for j in 1..=count {
            let angle = angles[i] * j as f64 / count as f64;
            path.samples.push(if j == count {
                end
            } else {
                center + (offset * angle.cos() + perpendicular * angle.sin())
            });
            path.distances
                .push(path.length + length * j as f64 / count as f64);
        }
        path.length += length;
        previous = end;
    }
    push_line(&mut path, previous, points[last]);
    path.points = points;
    Ok(path)
}

fn push_line(path: &mut LeadPath, start: Point, end: Point) {
    let length = start.distance(end);
    if length > 1e-10 {
        path.length += length;
        path.samples.push(end);
        path.distances.push(path.length);
    }
}

fn normalize_lead(points: &[Point]) -> DesignResult<Vec<Point>> {
    if points.len() < 2 || points.iter().any(|point| !point.is_finite()) {
        return Err("A flexible lead requires finite points and two endpoints".into());
    }
    let mut result: Vec<Point> = Vec::new();
    for &point in points {
        if result
            .last()
            .is_some_and(|previous| previous.distance(point) < 1e-10)
        {
            return Err("Flexible lead has duplicate or degenerate controls".into());
        }
        while result.len() >= 2 {
            let last = result.len() - 1;
            let before = result[last] - result[last - 1];
            let after = point - result[last];
            if before.dot(after) <= 0.0
                || before.cross(after).norm()
                    > 1e-10 * (before.dot(before) * after.dot(after)).sqrt()
            {
                break;
            }
            result.pop();
        }
        result.push(point);
    }
    Ok(result)
}

fn segment_approach(a: Point, b: Point, c: Point, d: Point) -> (f64, f64, f64) {
    let (u, v, w) = (b - a, d - c, a - c);
    let (aa, bb, cc, dd, ee) = (u.dot(u), u.dot(v), v.dot(v), u.dot(w), v.dot(w));
    let denominator = aa * cc - bb * bb;
    let mut s = if denominator > 1e-15 {
        ((bb * ee - cc * dd) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if cc > 1e-15 { (bb * s + ee) / cc } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if aa > 1e-15 {
            (-dd / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    } else if t > 1.0 {
        t = 1.0;
        s = if aa > 1e-15 {
            ((bb - dd) / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    ((a + u * s).distance(c + v * t), s, t)
}

fn lead_intersects_itself(path: &LeadPath, diameter: f64) -> bool {
    let segments: Vec<_> = path.samples.windows(2).collect();
    let local_length = PI * diameter + 2.0 * path.error;
    for (first, a) in segments.iter().enumerate() {
        for (second, b) in segments.iter().enumerate().skip(first + 2) {
            if path.distances[second + 1] - path.distances[first] <= local_length {
                continue;
            }
            let (distance, s, t) = segment_approach(a[0], a[1], b[0], b[1]);
            let along_first =
                path.distances[first] + s * (path.distances[first + 1] - path.distances[first]);
            let along_second =
                path.distances[second] + t * (path.distances[second + 1] - path.distances[second]);
            if along_second - along_first > local_length
                && distance < diameter + 2.0 * path.error - EPS
            {
                return true;
            }
        }
    }
    false
}

fn paths_intersect(a: &[Point], b: &[Point], distance: f64) -> bool {
    a.windows(2).any(|a| {
        b.windows(2)
            .any(|b| segment_approach(a[0], a[1], b[0], b[1]).0 < distance - EPS)
    })
}

#[derive(Clone, Copy)]
struct Body {
    center: Point,
    axes: Matrix,
    half: Point,
}

impl Body {
    fn enters(self, start: Point, end: Point, radius: f64) -> bool {
        let start = transform(self.axes, start - self.center);
        let end = transform(self.axes, end - self.center);
        let (mut lower, mut upper) = (0.0_f64, 1.0_f64);
        for i in 0..3 {
            let bound = (self.half.0[i] + radius - EPS).max(0.0);
            let delta = end.0[i] - start.0[i];
            if delta.abs() < 1e-14 {
                if !(-bound < start.0[i] && start.0[i] < bound) {
                    return false;
                }
                continue;
            }
            let a = (-bound - start.0[i]) / delta;
            let b = (bound - start.0[i]) / delta;
            lower = lower.max(a.min(b));
            upper = upper.min(a.max(b));
            if lower >= upper {
                return false;
            }
        }
        lower < upper && upper > EPS && lower < 1.0 - EPS
    }

    fn overlaps(self, other: Self) -> bool {
        let delta = other.center - self.center;
        let crosses = self
            .axes
            .iter()
            .flat_map(|a| other.axes.iter().map(|b| a.cross(*b)));
        self.axes
            .into_iter()
            .chain(other.axes)
            .chain(crosses)
            .all(|axis| {
                let norm = axis.norm();
                if norm < 1e-10 {
                    return true;
                }
                let axis = axis * (1.0 / norm);
                let extent = |body: Self| {
                    body.half
                        .0
                        .into_iter()
                        .zip(body.axes)
                        .map(|(half, direction)| half * direction.dot(axis).abs())
                        .sum::<f64>()
                };
                delta.dot(axis).abs() < extent(self) + extent(other) - EPS
            })
    }
}

struct ContactMechanics {
    point: Point,
    surface: f64,
    tip: Point,
    floor: f64,
    throat: f64,
}

impl ContactMechanics {
    fn fits(&self, start: Point, end: Point, radius: f64) -> bool {
        radius.is_finite()
            && radius > 0.0
            && start.is_finite()
            && end.is_finite()
            && (start.x() - end.x()).hypot(start.z() - end.z()) <= 1e-7
            && [start, end].into_iter().all(|point| {
                (point.x() - self.point.x()).abs() + radius <= self.throat + 1e-7
                    && (point.z() - self.point.z()).abs() + radius <= self.throat + 1e-7
            })
            && start.y().min(end.y()) >= self.floor - 1e-7
    }

    fn validate(&self, start: Point, end: Point, radius: f64) -> DesignResult<()> {
        let length = start.distance(end);
        let vertical_radius = if length > 1e-12 {
            radius
                * (1.0 - ((end.y() - start.y()) / length).powi(2))
                    .max(0.0)
                    .sqrt()
        } else {
            radius
        };
        if start.y().min(end.y()) - vertical_radius < self.surface - 1e-7
            && !self.fits(start, end, radius)
        {
            return Err(
                "Conductor intersects breadboard plastic outside its contact cavity".into(),
            );
        }
        Ok(())
    }
}

fn contact_mechanics(
    catalogs: &Catalogs,
    board: &Value,
    contact: &Value,
) -> DesignResult<ContactMechanics> {
    let mechanics = &catalogs.board_mechanics;
    let board_type = &board["type"];
    let board = catalogs
        .board(board)
        .filter(|_| list(&mechanics["moduleTypes"]).contains(board_type))
        .ok_or("Unknown breadboard mechanical contact")?;
    let point = coordinates(&contact["localPosition"]);
    let surface = board.deck_height;
    Ok(ContactMechanics {
        point,
        surface,
        tip: Point::new(
            point.x(),
            surface - number(&mechanics["insertionDepthMm"]) * MM_TO_WORLD,
            point.z(),
        ),
        floor: surface - number(&mechanics["cavityDepthMm"]) * MM_TO_WORLD,
        throat: number(&mechanics["throatHalfWidthWorld"]),
    })
}

pub(super) struct FlexibleAssembly {
    body: Body,
    leads: Vec<Vec<Point>>,
    radius: f64,
    version: u8,
}

pub(super) fn resolve_flexible(
    catalogs: &Catalogs,
    package: &Value,
    mount: &Value,
    board: &Value,
) -> DesignResult<FlexibleAssembly> {
    let profile = catalogs
        .flexible_profile(package)
        .filter(|profile| {
            profile["id"] == mount["mechanicalProfileId"]
                && super::same_revision(&profile["revision"], &mount["mechanicalProfileRevision"])
        })
        .ok_or("Unknown mechanical authoring profile revision")?;
    let version = mount["version"].as_f64().unwrap_or(0.0);
    if !matches!(version, 1.0 | 2.0) || mount["geometryRevision"].as_f64() != Some(version) {
        return Err("Unsupported flexible placement version".into());
    }
    let pins: HashSet<_> = list(&package["pins"])
        .iter()
        .map(|pin| text(&pin["id"]))
        .collect();
    if pins.len() != 2
        || list(&profile["pins"])
            .iter()
            .map(|pin| text(&pin["pinId"]))
            .collect::<HashSet<_>>()
            != pins
    {
        return Err("Flexible placement requires a declared two-lead strategy".into());
    }
    let bindings = list(&mount["contacts"]);
    let intents: HashMap<_, _> = list(&mount["leads"])
        .iter()
        .map(|lead| (text(&lead["pinId"]), list(&lead["controlPoints"])))
        .collect();
    if intents.len() != 2
        || list(&mount["leads"]).len() != 2
        || !pins.iter().all(|pin| intents.contains_key(pin))
    {
        return Err("Flexible placement must bind each stable pin exactly once".into());
    }
    let position = point(&mount["bodyPose"]["position"]);
    let angles = point(&mount["bodyPose"]["rotation"]);
    if !position.is_finite() || !angles.is_finite() {
        return Err("Flexible body pose must be finite".into());
    }
    let matrix = rotation(&angles);
    let half = point(&package["bodyMm"]) * (MM_TO_WORLD / 2.0);
    let body = Body {
        center: position,
        axes: transpose(matrix),
        half,
    };
    let contacts: HashMap<_, _> = bindings
        .iter()
        .map(|binding| {
            Ok((
                text(&binding["pinId"]),
                catalogs
                    .contact(board, &binding["contactId"])
                    .ok_or("Unknown mounting breadboard contact")?,
            ))
        })
        .collect::<Result<_, &str>>()?;
    if bindings[0]["contactId"] == bindings[1]["contactId"] {
        return Err("Flexible pins require distinct physical holes".into());
    }
    let anchors: Vec<_> = bindings
        .iter()
        .map(|binding| coordinates(&contacts[text(&binding["pinId"])]["localPosition"]))
        .collect();
    let plane = anchors[0].y().max(anchors[1].y());
    let low = position.y() - Point(matrix[1].0.map(f64::abs)).dot(half);
    if low < plane + number(&profile["bodyClearanceMm"]) * MM_TO_WORLD - EPS {
        return Err("Flexible body intersects the breadboard clearance".into());
    }
    if position.y() - plane > number(&profile["maxBodyLiftMm"]) * MM_TO_WORLD + EPS {
        return Err("Flexible body lift exceeds mechanical bounds".into());
    }
    let minimum = number(&profile["minBendRadiusMm"]) * MM_TO_WORLD;
    let preferred = if version == 1.0 {
        minimum
    } else if mount["routing"]["template"] == "axial-standing" {
        minimum.max(anchors[0].distance(anchors[1]) / 2.0)
    } else {
        profile
            .get("preferredBendRadiusMm")
            .map_or(minimum, |value| number(value) * MM_TO_WORLD)
    };
    let radius = lead_radius_mm(package) * MM_TO_WORLD;
    let exit_straight = number(&profile["exitStraightMm"]) * MM_TO_WORLD;
    let stub_height = number(&profile["contactStubMm"]) * MM_TO_WORLD;
    let mut leads = Vec::new();
    for pin in list(&profile["pins"]) {
        let id = text(&pin["pinId"]);
        let controls = intents[id];
        if controls.len() as f64 > number(&profile["maxControlsPerLead"]) {
            return Err("Too many flexible lead controls".into());
        }
        let exit = point(&pin["bodyExitMm"]);
        let outward = if profile["strategy"] == "axial" {
            Point::new(1.0_f64.copysign(exit.x()), 0.0, 0.0)
        } else {
            Point::new(0.0, -1.0, 0.0)
        };
        let exit = position + transform(matrix, exit * MM_TO_WORLD);
        let shoulder = exit + transform(matrix, outward * exit_straight);
        let contact = contacts[id];
        let anchor = coordinates(&contact["localPosition"]);
        let stub = anchor + Point::new(0.0, stub_height, 0.0);
        let mut raw = vec![exit, shoulder];
        raw.extend(controls.iter().map(point));
        raw.extend([stub, anchor]);
        if raw
            .iter()
            .any(|point| !point.is_finite() || point.y() < plane - EPS)
        {
            return Err("Flexible lead intersects the breadboard or is nonfinite".into());
        }
        let straights = if version == 1.0 {
            [0.0, 0.0]
        } else {
            [exit_straight, stub_height]
        };
        let mut path = build_lead_path(&raw, minimum, preferred, straights)?;
        let min_length = number(&profile["minLeadLengthMm"]);
        let max_length = number(&profile["maxLeadLengthMm"]);
        if !(min_length..=max_length).contains(&(path.length / MM_TO_WORLD)) {
            return Err("Flexible lead length exceeds mechanical bounds".into());
        }
        if path
            .samples
            .windows(2)
            .any(|segment| body.enters(segment[0], segment[1], 0.0))
        {
            return Err("Flexible lead intersects its rigid body".into());
        }
        if version == 2.0 {
            let radius = radius + path.error;
            if path
                .samples
                .windows(2)
                .skip(1)
                .any(|segment| body.enters(segment[0], segment[1], radius))
            {
                return Err("Flexible lead tube intersects its rigid body".into());
            }
            if path.samples[..path.samples.len() - 1]
                .iter()
                .any(|point| point.y() - radius < plane - EPS)
            {
                return Err("Flexible lead tube intersects the breadboard".into());
            }
        }
        if lead_intersects_itself(&path, radius * 2.0) {
            return Err("Flexible lead intersects itself".into());
        }
        if let Some(insertion) = package.get("boardInsertion") {
            let mechanics = contact_mechanics(catalogs, board, contact)?;
            if !mechanics.fits(anchor, mechanics.tip, radius) {
                return Err("Conductor does not fit its contact cavity".into());
            }
            let extra = anchor.y() - mechanics.tip.y();
            if (path.length + extra) / MM_TO_WORLD
                > max_length.min(number(&insertion["maxExternalLeadMm"])) + 1e-6
            {
                return Err("The declared metal lead is too short for this board insertion".into());
            }
            let mut points = path.points;
            points.push(mechanics.tip);
            let straights = if version == 1.0 {
                straights
            } else {
                [exit_straight, stub_height + extra]
            };
            path = build_lead_path(&points, minimum, preferred, straights)?;
            for segment in path.samples.windows(2) {
                mechanics.validate(segment[0], segment[1], radius)?;
            }
        }
        leads.push(path);
    }
    if paths_intersect(
        &leads[0].samples,
        &leads[1].samples,
        radius * 2.0 + leads[0].error + leads[1].error,
    ) {
        return Err("Flexible leads intersect each other".into());
    }

    let board_yaw = yaw_matrix(module_yaw(board));
    let board_position = translation(board);
    let world = |point| board_position + transform(board_yaw, point);
    Ok(FlexibleAssembly {
        body: Body {
            center: world(position),
            axes: body.axes.map(|axis| transform(board_yaw, axis)),
            half,
        },
        leads: leads
            .into_iter()
            .map(|path| path.samples.into_iter().map(world).collect())
            .collect(),
        radius: radius + (minimum / 512.0).min(1e-5),
        version: version as u8,
    })
}

#[derive(Clone, Copy)]
struct Segment {
    start: Point,
    end: Point,
    radius: f64,
}

impl Segment {
    fn hits(self, other: Self) -> bool {
        segment_approach(self.start, self.end, other.start, other.end).0
            < self.radius + other.radius - EPS
    }
}

fn rigid_segments(package: &Value, pin: &Value, insertion: f64) -> DesignResult<Vec<Segment>> {
    let shape = &package["shape"];
    let section = &shape["leadSection"];
    let definition = list(&shape["leadExits"])
        .iter()
        .find(|exit| exit["pinId"] == pin["id"])
        .ok_or("A shaped rigid lead requires its declared body exit")?;
    let center_y = number(&package["standoffMm"]) + number(&package["bodyMm"]["y"]) / 2.0;
    let exit = point(&definition["positionMm"]) + Point::new(0.0, center_y, 0.0);
    let tip = point(&pin["positionMm"]);
    let width = number(&section["widthMm"]);
    let thickness = number(&section["thicknessMm"]);
    let insertion_mm = insertion / MM_TO_WORLD;
    if !matches!(shape["family"].as_str(), Some("dip" | "to92")) {
        return fixed_segments(package, exit, tip, point(&definition["normal"]), insertion);
    }
    let mut stations = Vec::new();
    if shape["family"] == "dip" {
        let shoulder = section.get("shoulderWidthMm").map_or(width, number);
        let normal_z = number(&definition["normal"]["z"]);
        let sign = if normal_z != 0.0 {
            normal_z.signum()
        } else if tip.z() >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let radius = 0.42_f64
            .min((tip.z() - exit.z()).abs() * 0.75)
            .min((exit.y() - tip.y()) * 0.3);
        stations.push((exit, shoulder));
        let bend_start = Point::new(exit.x(), exit.y(), tip.z() - sign * radius);
        if exit.distance(bend_start) > 1e-9 / MM_TO_WORLD {
            stations.push((bend_start, shoulder));
        }
        for i in 1..=12 {
            let theta = i as f64 / 12.0 * PI / 2.0;
            stations.push((
                Point::new(
                    exit.x(),
                    exit.y() - radius * (1.0 - theta.cos()),
                    bend_start.z() + sign * radius * theta.sin(),
                ),
                shoulder,
            ));
        }
        let end_y = stations.last().expect("bend stations").0.y();
        let insertable = package.get("boardInsertion").is_some();
        let narrow_start = (tip.y() + if insertable { 1.05 } else { 0.6 }).max(end_y - 0.35);
        let narrow_end = (tip.y() + if insertable { 0.8 } else { 0.35 }).max(narrow_start - 0.65);
        if narrow_start < end_y - 1e-9 / MM_TO_WORLD {
            stations.push((Point::new(tip.x(), narrow_start, tip.z()), shoulder));
        }
        stations.push((Point::new(tip.x(), narrow_end, tip.z()), width));
        stations.push((tip, width));
    } else {
        let insertable = package.get("boardInsertion").is_some();
        let height = exit.y() - tip.y() - if insertable { 1.0 } else { 0.0 };
        stations.push((exit, width));
        for i in 1..=24 {
            let t = i as f64 / 24.0;
            let blend = t * t * (3.0 - 2.0 * t);
            stations.push((
                Point::new(
                    exit.x() + (tip.x() - exit.x()) * blend,
                    exit.y() - height * t,
                    exit.z() + (tip.z() - exit.z()) * blend,
                ),
                width,
            ));
        }
        if insertable {
            stations.push((tip, width));
        }
    }
    if insertion_mm > 0.0 {
        stations.push((tip - Point::new(0.0, insertion_mm, 0.0), width));
    }
    lead_budget(
        package,
        stations
            .windows(2)
            .map(|pair| pair[0].0.distance(pair[1].0))
            .sum(),
    )?;
    Ok(stations
        .windows(2)
        .map(|pair| Segment {
            start: pair[0].0 * MM_TO_WORLD,
            end: pair[1].0 * MM_TO_WORLD,
            radius: pair[0].1.hypot(thickness).max(pair[1].1.hypot(thickness)) / 2.0 * MM_TO_WORLD,
        })
        .collect())
}

fn fixed_segments(
    package: &Value,
    exit: Point,
    tip: Point,
    normal: Point,
    insertion: f64,
) -> DesignResult<Vec<Segment>> {
    let section = &package["shape"]["leadSection"];
    let exit = exit * MM_TO_WORLD;
    let anchor = tip * MM_TO_WORLD;
    let outward = normal.unit();
    let offset = (exit.x() - anchor.x()).hypot(exit.z() - anchor.z());
    let radial = outward.y() < -0.999 && offset > 1e-8;
    let stamped = section["kind"] == "stamped";
    let straight = (if stamped || radial { 0.15 } else { 0.4 }) * MM_TO_WORLD;
    let round = section["kind"] == "round";
    let width = number(&section["widthMm"]);
    let minimum = (if stamped {
        0.15
    } else if round && width <= 0.5 {
        0.3
    } else {
        0.4
    }) * MM_TO_WORLD;
    let mut points = if radial {
        let mid_y = exit.y() - straight - minimum;
        vec![
            exit,
            Point::new(exit.x(), mid_y, exit.z()),
            Point::new(anchor.x(), mid_y, anchor.z()),
            anchor,
        ]
    } else {
        let shoulder = exit + outward * straight * if stamped { 2.0 } else { 1.0 };
        let mut points = vec![exit, shoulder];
        if offset > 1e-8 {
            points.push(Point::new(anchor.x(), exit.y(), anchor.z()));
        }
        points.extend([anchor + Point::new(0.0, straight, 0.0), anchor]);
        points
    };
    if insertion > 0.0 {
        points.push(anchor - Point::new(0.0, insertion, 0.0));
    }
    let preferred = if stamped || radial {
        minimum
    } else {
        0.7 * MM_TO_WORLD
    };
    let path = build_lead_path(&points, minimum, preferred, [straight; 2])?;
    lead_budget(package, path.length / MM_TO_WORLD)?;
    let thickness = if round {
        width
    } else {
        width.hypot(number(&section["thicknessMm"]))
    };
    let radius = thickness / 2.0 * MM_TO_WORLD + path.error;
    Ok(path
        .samples
        .windows(2)
        .map(|pair| Segment {
            start: pair[0],
            end: pair[1],
            radius,
        })
        .collect())
}

fn lead_budget(package: &Value, length: f64) -> DesignResult<()> {
    if let Some(insertion) = package.get("boardInsertion") {
        if length > number(&insertion["maxExternalLeadMm"]) + 1e-6 {
            return Err("The declared metal lead is too short for this board insertion".into());
        }
    }
    Ok(())
}

fn rigid_world_pose(module: &Value, by_id: &HashMap<&str, &Value>) -> (Point, f64) {
    if let Some(mount) = module.get("mount") {
        let board = by_id[text(&mount["boardId"])];
        let board_yaw = board.get("rotation").map_or(0.0, wrapped_yaw);
        let pose = &mount["pose"];
        (
            translation(board) + yaw(board_yaw, point(&pose["position"])),
            combined_rigid_yaw(board, &pose["rotation"]),
        )
    } else {
        (
            translation(module),
            module.get("rotation").map_or(0.0, wrapped_yaw),
        )
    }
}

fn rigid_body(
    module: &Value,
    package: &Value,
    by_id: &HashMap<&str, &Value>,
) -> DesignResult<Body> {
    let (position, rotation) = rigid_world_pose(module, by_id);
    if !position.is_finite() || !rotation.is_finite() {
        return Err("Physical world pose is outside supported numerical bounds".into());
    }
    let half = point(&package["bodyMm"]) * (MM_TO_WORLD / 2.0);
    let low = position.y() + number(&package["standoffMm"]) * MM_TO_WORLD;
    Ok(Body {
        center: Point::new(position.x(), low + half.y(), position.z()),
        axes: transpose(yaw_matrix(rotation)),
        half,
    })
}

fn rigid_world_leads(
    catalogs: &Catalogs,
    module: &Value,
    package: &Value,
    by_id: &HashMap<&str, &Value>,
) -> DesignResult<Vec<Segment>> {
    let mount = module.get("mount");
    let mut insertion = 0.0;
    if let Some(mount) = mount.filter(|_| package.get("boardInsertion").is_some()) {
        let board = by_id[text(&mount["boardId"])];
        let section = &package["shape"]["leadSection"];
        let width = number(&section["widthMm"]);
        let diameter = if section["kind"] == "round" {
            width
        } else {
            width.hypot(number(&section["thicknessMm"]))
        };
        for binding in list(&mount["contacts"]) {
            let contact = catalogs
                .contact(board, &binding["contactId"])
                .ok_or("Unknown mounting breadboard contact")?;
            let mechanics = contact_mechanics(catalogs, board, contact)?;
            if !mechanics.fits(mechanics.point, mechanics.tip, diameter / 2.0 * MM_TO_WORLD) {
                return Err("Conductor does not fit its contact cavity".into());
            }
            insertion = mechanics.point.y() - mechanics.tip.y();
        }
    }
    if package.get("shape").is_none() {
        return Ok(Vec::new());
    }
    let (position, angle) = rigid_world_pose(module, by_id);
    let matrix = yaw_matrix(angle);
    let world = |point| position + transform(matrix, point);
    let mut result = Vec::new();
    for pin in list(&package["pins"]) {
        let segments = rigid_segments(package, pin, insertion)?;
        if let Some(mount) = mount.filter(|_| insertion != 0.0) {
            let board = by_id[text(&mount["boardId"])];
            let binding = list(&mount["contacts"])
                .iter()
                .find(|binding| binding["pinId"] == pin["id"])
                .expect("validated pin bindings");
            let contact = catalogs
                .contact(board, &binding["contactId"])
                .ok_or("Unknown mounting breadboard contact")?;
            let mechanics = contact_mechanics(catalogs, board, contact)?;
            let position = point(&mount["pose"]["position"]);
            let matrix = yaw_matrix(number(&mount["pose"]["rotation"]));
            let local = |point| position + transform(matrix, point);
            for segment in &segments {
                mechanics.validate(local(segment.start), local(segment.end), segment.radius)?;
            }
        }
        result.extend(segments.into_iter().map(|segment| Segment {
            start: world(segment.start),
            end: world(segment.end),
            ..segment
        }));
    }
    Ok(result)
}

#[derive(Clone, Copy)]
struct Bounds([(f64, f64); 3]);

impl Bounds {
    fn body(body: Body) -> Self {
        Self(std::array::from_fn(|i| {
            let extent = body
                .axes
                .iter()
                .enumerate()
                .map(|(j, axis)| axis.0[i].abs() * body.half.0[j])
                .sum::<f64>();
            (body.center.0[i] - extent, body.center.0[i] + extent)
        }))
    }

    fn segments(segments: &[Segment]) -> Self {
        let mut bounds = [(f64::INFINITY, f64::NEG_INFINITY); 3];
        for segment in segments {
            for (i, (low, high)) in bounds.iter_mut().enumerate() {
                *low = low.min(segment.start.0[i].min(segment.end.0[i]) - segment.radius);
                *high = high.max(segment.start.0[i].max(segment.end.0[i]) + segment.radius);
            }
        }
        Self(bounds)
    }

    fn overlaps(self, other: Self) -> bool {
        self.0
            .into_iter()
            .zip(other.0)
            .all(|((a0, a1), (b0, b1))| a0 < b1 - EPS && b0 < a1 - EPS)
    }
}

pub(super) fn collisions(
    catalogs: &Catalogs,
    modules: &[Value],
    parts: &[Option<&Value>],
    by_id: &HashMap<&str, &Value>,
    flexible: &HashMap<&str, FlexibleAssembly>,
) -> DesignResult<()> {
    let mut bodies = Vec::new();
    for (module, part) in modules.iter().zip(parts) {
        if let Some(part) = part {
            let id = text(&module["id"]);
            let body = match flexible.get(id) {
                Some(assembly) => assembly.body,
                None => rigid_body(module, &part["package"], by_id)?,
            };
            if !body.center.is_finite() || body.axes.iter().any(|axis| !axis.is_finite()) {
                return Err("Physical world pose is outside supported numerical bounds".into());
            }
            bodies.push((id, body, Bounds::body(body), module.get("mount").is_some()));
        }
    }
    for (index, (_, body, _, mounted)) in bodies.iter().enumerate() {
        for (_, other, _, other_mounted) in &bodies[index + 1..] {
            if (*mounted || *other_mounted) && body.overlaps(*other) {
                return Err("Physical component bodies overlap".into());
            }
        }
    }

    let ordered_flexible: Vec<_> = modules
        .iter()
        .filter_map(|module| {
            let id = text(&module["id"]);
            flexible.get(id).map(|assembly| (id, assembly))
        })
        .collect();
    for (id, assembly) in &ordered_flexible {
        if assembly
            .leads
            .iter()
            .flatten()
            .any(|point| !point.is_finite())
        {
            return Err("Physical world pose is outside supported numerical bounds".into());
        }
        for (other, body, _, _) in &bodies {
            if other != id
                && assembly.leads.iter().any(|lead| {
                    lead.windows(2)
                        .any(|pair| body.enters(pair[0], pair[1], assembly.radius))
                })
            {
                return Err("Flexible lead intersects another component body".into());
            }
        }
    }
    for (index, (_, a)) in ordered_flexible.iter().enumerate() {
        for (_, b) in &ordered_flexible[index + 1..] {
            if a.version == 1 && b.version == 1 {
                continue;
            }
            if a.leads.iter().any(|a_lead| {
                b.leads
                    .iter()
                    .any(|b_lead| paths_intersect(a_lead, b_lead, a.radius + b.radius))
            }) {
                return Err("Flexible leads intersect another component lead".into());
            }
        }
    }

    let mut rigid = Vec::new();
    for (module, part) in modules.iter().zip(parts) {
        let id = text(&module["id"]);
        let Some(part) = part.filter(|_| !flexible.contains_key(id)) else {
            continue;
        };
        let leads = rigid_world_leads(catalogs, module, &part["package"], by_id)?;
        if !leads.is_empty() {
            let bounds = Bounds::segments(&leads);
            rigid.push((id, leads, bounds));
        }
    }
    let flexible_segments: Vec<_> = ordered_flexible
        .iter()
        .map(|(id, assembly)| {
            let segments: Vec<_> = assembly
                .leads
                .iter()
                .flat_map(|lead| {
                    lead.windows(2).map(|pair| Segment {
                        start: pair[0],
                        end: pair[1],
                        radius: assembly.radius,
                    })
                })
                .collect();
            let bounds = Bounds::segments(&segments);
            (*id, segments, bounds)
        })
        .collect();
    for (id, leads, bounds) in &rigid {
        for (other, body, other_bounds, _) in &bodies {
            if other != id
                && bounds.overlaps(*other_bounds)
                && leads
                    .iter()
                    .any(|segment| body.enters(segment.start, segment.end, segment.radius))
            {
                return Err("Rigid lead intersects another component body".into());
            }
        }
        for (other, other_leads, other_bounds) in &flexible_segments {
            if other != id
                && bounds.overlaps(*other_bounds)
                && leads
                    .iter()
                    .any(|segment| other_leads.iter().any(|other| segment.hits(*other)))
            {
                return Err("Rigid lead intersects another component lead".into());
            }
        }
    }
    for (index, (_, leads, bounds)) in rigid.iter().enumerate() {
        for (_, other_leads, other_bounds) in &rigid[index + 1..] {
            if bounds.overlaps(*other_bounds)
                && leads
                    .iter()
                    .any(|segment| other_leads.iter().any(|other| segment.hits(*other)))
            {
                return Err("Rigid leads intersect another component lead".into());
            }
        }
    }
    Ok(())
}
