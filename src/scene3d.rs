//! Minimal 3D geometry for the sensor map.
//!
//! The box is centered at the origin with `SceneDims` dimensions
//! (width X, height Y, depth Z). The projection is perspective; the
//! camera orbits around the object via yaw (`yaw`) and pitch
//! (`pitch`), with `zoom` that adjusts the scale.
//!
//! The module is independent from egui: it returns screen coordinates as
//! `(x, y)` pairs with `y` pointing down, so that it can be reused both
//! by the interactive view and by the PDF export.

use crate::models::{SceneDims, SensorPos};

/// Essential 3D vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    pub fn scale(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Vec3 {
        let l = self.length();
        if l > 1e-9 {
            self.scale(1.0 / l)
        } else {
            self
        }
    }
}

/// Orbital camera parameters.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Rotation around the vertical axis, in radians.
    pub yaw: f32,
    /// Vertical tilt, in radians (limited to ±~85°).
    pub pitch: f32,
    /// Magnification factor.
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.65,
            pitch: 0.42,
            zoom: 1.0,
        }
    }
}

/// Number of faces of the box.
pub const FACE_COUNT: usize = 6;

/// Outward normal of face `face` (0..5).
pub fn face_normal(face: usize) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, 0.0, 0.0),
        1 => Vec3::new(-1.0, 0.0, 0.0),
        2 => Vec3::new(0.0, 1.0, 0.0),
        3 => Vec3::new(0.0, -1.0, 0.0),
        4 => Vec3::new(0.0, 0.0, 1.0),
        _ => Vec3::new(0.0, 0.0, -1.0),
    }
}

/// Distance of the origin from the plane containing face `face`
/// (the plane equation is `normal · p = offset`).
fn face_plane_offset(face: usize, dims: SceneDims) -> f32 {
    match face {
        0 | 1 => dims.w * 0.5,
        2 | 3 => dims.h * 0.5,
        _ => dims.d * 0.5,
    }
}

/// Local point on face `face` for coordinates `u`, `v` in `[0, 1]`.
pub fn face_point(face: usize, u: f32, v: f32, dims: SceneDims) -> Vec3 {
    let (hx, hy, hz) = (dims.w * 0.5, dims.h * 0.5, dims.d * 0.5);
    match face {
        0 => Vec3::new(hx, -hy + v * 2.0 * hy, -hz + u * 2.0 * hz),
        1 => Vec3::new(-hx, -hy + v * 2.0 * hy, hz - u * 2.0 * hz),
        2 => Vec3::new(-hx + u * 2.0 * hx, hy, -hz + v * 2.0 * hz),
        3 => Vec3::new(-hx + u * 2.0 * hx, -hy, hz - v * 2.0 * hz),
        4 => Vec3::new(-hx + u * 2.0 * hx, -hy + v * 2.0 * hy, hz),
        _ => Vec3::new(hx - u * 2.0 * hx, -hy + v * 2.0 * hy, -hz),
    }
}

/// Inverse of [`face_point`]: `(u, v)` coordinates on the face for a local point.
pub fn face_uv(face: usize, p: Vec3, dims: SceneDims) -> (f32, f32) {
    let (hx, hy, hz) = (dims.w * 0.5, dims.h * 0.5, dims.d * 0.5);
    let (u, v) = match face {
        0 => ((p.z + hz) / (2.0 * hz), (p.y + hy) / (2.0 * hy)),
        1 => ((hz - p.z) / (2.0 * hz), (p.y + hy) / (2.0 * hy)),
        2 => ((p.x + hx) / (2.0 * hx), (p.z + hz) / (2.0 * hz)),
        3 => ((p.x + hx) / (2.0 * hx), (hz - p.z) / (2.0 * hz)),
        4 => ((p.x + hx) / (2.0 * hx), (p.y + hy) / (2.0 * hy)),
        _ => ((hx - p.x) / (2.0 * hx), (p.y + hy) / (2.0 * hy)),
    };
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Perspective projector: converts local points into screen coordinates.
#[derive(Clone, Copy)]
pub struct Projector {
    yaw: f32,
    pitch: f32,
    cx: f32,
    cy: f32,
    focal: f32,
    dist: f32,
}

impl Projector {
    /// Creates a projector with center `(cx, cy)` and `viewport` = the useful side
    /// of the drawing area (in pixels or millimeters: the units stay consistent).
    pub fn new(cx: f32, cy: f32, viewport: f32, cam: &Camera, dims: SceneDims) -> Self {
        let radius = dims.half_diagonal().max(0.25);
        let dist = radius * 2.8 + 0.4;
        let focal = viewport.max(60.0) * cam.zoom;
        Self {
            yaw: cam.yaw,
            pitch: cam.pitch,
            cx,
            cy,
            focal,
            dist,
        }
    }

    /// Applies the camera rotation (yaw + pitch).
    pub fn rotate(&self, p: Vec3) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let x1 = cy * p.x + sy * p.z;
        let z1 = -sy * p.x + cy * p.z;
        let (sp, cp) = self.pitch.sin_cos();
        let y2 = cp * p.y - sp * z1;
        let z2 = sp * p.y + cp * z1;
        Vec3::new(x1, y2, z2)
    }

    /// Inverse of the camera rotation.
    pub fn rotate_inv(&self, q: Vec3) -> Vec3 {
        let (sp, cp) = self.pitch.sin_cos();
        let y1 = cp * q.y + sp * q.z;
        let z1 = -sp * q.y + cp * q.z;
        let (sy, cy) = self.yaw.sin_cos();
        let x = cy * q.x - sy * z1;
        let z = sy * q.x + cy * z1;
        Vec3::new(x, y1, z)
    }

    /// Projects a local point into screen coordinates `(x, y)` with `y` at the bottom.
    pub fn project(&self, p: Vec3) -> (f32, f32) {
        let q = self.rotate(p);
        let dz = (self.dist - q.z).max(1e-4);
        (self.cx + self.focal * q.x / dz, self.cy - self.focal * q.y / dz)
    }

    /// Distance from the camera (smaller values = closer).
    pub fn depth(&self, p: Vec3) -> f32 {
        self.dist - self.rotate(p).z
    }

    /// Local point that projects exactly onto `(sx, sy)` **while keeping the
    /// same distance from the camera** as `p`.
    ///
    /// It is the free drag: the point slides in the plane parallel to the
    /// view that passes through `p` and is not brought back onto any wall, so
    /// it can detach from the container or enter it. Since that plane is oblique
    /// with respect to the box whenever the camera looks from three quarters, the
    /// movement that results has components on `x`, `y` and `z` together: while
    /// dragging, all three axes are used, not only the two of the
    /// wall.
    ///
    /// The depth is the initial one and stays fixed, so the point neither
    /// approaches nor moves away while it moves.
    ///
    /// If the camera looks exactly along the axes, the drag plane
    /// coincides with a coordinate plane and a single dimension
    /// stays unchanged: it is the geometry, not a limit of the function.
    pub fn drag_at_depth(&self, p: Vec3, sx: f32, sy: f32) -> Vec3 {
        let q = self.rotate(p);
        let dz = (self.dist - q.z).max(1e-4);
        let qx = (sx - self.cx) * dz / self.focal;
        let qy = -(sy - self.cy) * dz / self.focal;
        self.rotate_inv(Vec3::new(qx, qy, q.z))
    }

    /// Reconstructs the ray (origin, direction) in local coordinates starting
    /// from a screen point.
    pub fn unproject(&self, sx: f32, sy: f32) -> (Vec3, Vec3) {
        let ax = (sx - self.cx) / self.focal;
        let ay = -(sy - self.cy) / self.focal;
        let origin = self.rotate_inv(Vec3::new(0.0, 0.0, self.dist));
        let dir = self.rotate_inv(Vec3::new(ax, ay, -1.0)).normalized();
        (origin, dir)
    }
}

/// A projected face, ready for drawing.
#[derive(Clone, Copy)]
pub struct FaceQuad {
    /// The four projected corners, in the order (0,0) (1,0) (1,1) (0,1).
    pub pts: [(f32, f32); 4],
    /// Average depth (for sorting from the back toward the camera).
    pub depth: f32,
    /// Light factor 0.45..1.0 computed from the face normal.
    pub shade: f32,
}

/// Projects the six faces and sorts them from the farthest to the nearest.
pub fn face_quads(proj: &Projector, dims: SceneDims) -> Vec<FaceQuad> {
    // Fixed light in camera space (from the top left, toward
    // the observer) so that the front faces appear lit.
    let light = Vec3::new(-0.35, 0.45, 0.82).normalized();
    let mut out: Vec<FaceQuad> = Vec::with_capacity(FACE_COUNT);
    let corners = [(0.0f32, 0.0f32), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

    for face in 0..FACE_COUNT {
        let mut pts = [(0.0f32, 0.0f32); 4];
        let mut depth = 0.0;
        for (i, (u, v)) in corners.iter().enumerate() {
            let p = face_point(face, *u, *v, dims);
            pts[i] = proj.project(p);
            depth += proj.depth(p);
        }
        let n_rot = proj.rotate(face_normal(face));
        let shade = 0.45 + 0.55 * n_rot.dot(light).max(0.0);
        out.push(FaceQuad {
            pts,
            depth: depth / 4.0,
            shade,
        });
    }

    out.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Finds the face hit by the ray `origin + t·dir` and returns
/// `(face, u, v)`. The face nearest to the camera is chosen.
pub fn pick_face(origin: Vec3, dir: Vec3, dims: SceneDims) -> Option<(usize, f32, f32)> {
    let h = Vec3::new(dims.w * 0.5, dims.h * 0.5, dims.d * 0.5);
    let mut best: Option<(usize, f32, Vec3)> = None;

    for face in 0..FACE_COUNT {
        let nrm = face_normal(face);
        let offset = face_plane_offset(face, dims);
        let denom = nrm.dot(dir);
        if denom.abs() < 1e-9 {
            continue;
        }
        let t = (offset - nrm.dot(origin)) / denom;
        if t <= 1e-4 {
            continue;
        }
        let hit = origin.add(dir.scale(t));
        let inside = match face {
            0 | 1 => hit.y.abs() <= h.y + 1e-4 && hit.z.abs() <= h.z + 1e-4,
            2 | 3 => hit.x.abs() <= h.x + 1e-4 && hit.z.abs() <= h.z + 1e-4,
            _ => hit.x.abs() <= h.x + 1e-4 && hit.y.abs() <= h.y + 1e-4,
        };
        if !inside {
            continue;
        }
        if best.map_or(true, |(_, bt, _)| t < bt) {
            best = Some((face, t, hit));
        }
    }

    best.map(|(face, _, hit)| {
        let (u, v) = face_uv(face, hit, dims);
        (face, u, v)
    })
}

/// 3D (local) position of a saved sensor.
///
/// A sensor never dragged lies on wall `face` at coordinates `u`, `v`
/// and follows the container when its dimensions change. A dragged one is
/// stored in normalized coordinates (fractions of the half dimension) and is
/// brought back to meters by multiplying by the current half dimensions: so even
/// sensors detached from the wall adapt to the new sizes.
pub fn sensor_point(pos: &SensorPos, dims: SceneDims) -> Vec3 {
    match pos.pos {
        Some(p) => Vec3::new(
            p.x * dims.w * 0.5,
            p.y * dims.h * 0.5,
            p.z * dims.d * 0.5,
        ),
        None => face_point(pos.face, pos.u, pos.v, dims),
    }
}

/// Converts an absolute local position into normalized coordinates, that is
/// the inverse operation of [`sensor_point`] for dragged sensors.
///
/// Returns fractions of the half dimension: the face `x = width/2`
/// corresponds to `x = 1.0`, the origin to `0.0`.
pub fn normalize_point(p: Vec3, dims: SceneDims) -> Vec3 {
    Vec3::new(
        p.x / (dims.w * 0.5),
        p.y / (dims.h * 0.5),
        p.z / (dims.d * 0.5),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dims() -> SceneDims {
        SceneDims {
            w: 6.0,
            h: 2.5,
            d: 2.4,
        }
    }

    #[test]
    fn punto_faccia_e_uv_sono_inverse() {
        let d = dims();
        for face in 0..FACE_COUNT {
            for (u, v) in [(0.0, 0.0), (0.25, 0.75), (1.0, 1.0), (0.5, 0.5)] {
                let p = face_point(face, u, v, d);
                let (u2, v2) = face_uv(face, p, d);
                assert!((u - u2).abs() < 1e-4, "u faccia {face}");
                assert!((v - v2).abs() < 1e-4, "v faccia {face}");
            }
        }
    }

    #[test]
    fn picking_trova_la_faccia_piu_vicina() {
        let d = dims();
        // Ray along -Z: it hits the +Z face (index 4).
        let origin = Vec3::new(0.5, 0.0, 10.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        let (face, u, v) = pick_face(origin, dir, d).expect("il raggio colpisce il solido");
        assert_eq!(face, 4);
        // u along +X: (0.5 + 3.0) / 6.0 = 0.5833…
        assert!((u - 0.583_333).abs() < 1e-3, "u = {u}");
        assert!((v - 0.5).abs() < 1e-3, "v = {v}");
    }

    #[test]
    fn raggio_che_manca_non_colpisce() {
        let (o, dir) = (Vec3::new(100.0, 0.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(pick_face(o, dir, dims()).is_none());
    }

    #[test]
    fn proiezione_e_unproiezione_coerenti() {
        let d = dims();
        let cam = Camera::default();
        let proj = Projector::new(400.0, 300.0, 600.0, &cam, d);
        let p = face_point(4, 0.4, 0.7, d);
        let (sx, sy) = proj.project(p);
        let (o, dir) = proj.unproject(sx, sy);
        let (face, u, v) = pick_face(o, dir, d).expect("ricolpisce il punto");
        assert_eq!(face, 4);
        assert!((u - 0.4).abs() < 1e-3, "u = {u}");
        assert!((v - 0.7).abs() < 1e-3, "v = {v}");
    }

    /// The free drag brings the point exactly under the cursor and does not
    /// change its distance from the camera. There is no wall constraint,
    /// so it also holds when the point is already detached from the container.
    #[test]
    fn trascinamento_libero_segue_il_cursore_senza_cambiare_profondita() {
        let d = dims();

        for (yaw, pitch) in [(0.0f32, 0.0f32), (0.65, 0.42), (1.2, 0.7), (0.65, -0.42)] {
            let cam = Camera { yaw, pitch, zoom: 1.0 };
            let proj = Projector::new(400.0, 300.0, 400.0, &cam, d);

            for face in 0..FACE_COUNT {
                for (u0, v0) in [(0.4_f32, 0.7_f32), (0.6, 0.3)] {
                    let p0 = face_point(face, u0, v0, d);
                    let (sx, sy) = proj.project(p0);
                    let d0 = proj.depth(p0);

                    for (dx, dy) in [(0.0_f32, 0.0), (40.0, 0.0), (0.0, 40.0), (-40.0, -40.0)] {
                        let p = proj.drag_at_depth(p0, sx + dx, sy + dy);

                        // It falls exactly where one points...
                        let (ax, ay) = proj.project(p);
                        let scarto = ((ax - sx - dx).powi(2) + (ay - sy - dy).powi(2)).sqrt();
                        assert!(
                            scarto < 0.5,
                            "yaw={yaw} pitch={pitch} faccia={face} delta=({dx}, {dy}): \
                             il punto cade a {scarto:.2}px dal cursore"
                        );

                        // ...without approaching or moving away from the camera.
                        assert!(
                            (proj.depth(p) - d0).abs() < 1e-3,
                            "yaw={yaw} pitch={pitch} faccia={face}: profondità {} invece di {d0}",
                            proj.depth(p)
                        );
                    }
                }
            }
        }
    }

    /// With the camera from three quarters the drag plane is oblique
    /// with respect to the container: moving the cursor the point shifts with
    /// components on all three directions, not only on the two axes of the
    /// wall it lay on.
    #[test]
    fn trascinamento_libero_agisce_su_tutti_e_tre_gli_assi() {
        let d = dims();
        let cam = Camera {
            yaw: 0.65,
            pitch: 0.42,
            zoom: 1.0,
        };
        let proj = Projector::new(400.0, 300.0, 400.0, &cam, d);

        let p0 = face_point(4, 0.5, 0.5, d);
        let (sx, sy) = proj.project(p0);

        // Two orthogonal directions on screen generate the movement plane.
        let direzioni = [
            proj.drag_at_depth(p0, sx + 60.0, sy).add(p0.scale(-1.0)),
            proj.drag_at_depth(p0, sx, sy + 60.0).add(p0.scale(-1.0)),
        ];
        let raggio = direzioni
            .iter()
            .fold(0.0_f32, |a, v| a.max((v.x * v.x + v.y * v.y + v.z * v.z).sqrt()));
        assert!(raggio > 0.5, "spostamento troppo piccolo: {raggio}");

        // None of the three directions must stay out of reach.
        for (asse, componente) in [
            (
                "x",
                direzioni.iter().fold(0.0_f32, |a, v| a.max(v.x.abs())),
            ),
            (
                "y",
                direzioni.iter().fold(0.0_f32, |a, v| a.max(v.y.abs())),
            ),
            (
                "z",
                direzioni.iter().fold(0.0_f32, |a, v| a.max(v.z.abs())),
            ),
        ] {
            assert!(
                componente > raggio * 0.1,
                "l'asse {asse} è appena raggiungibile: {componente} contro un raggio di {raggio}"
            );
        }
    }
}
