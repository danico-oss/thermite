//! Geometria 3D minimale per la mappa dei sensori.
//!
//! Il parallelepipedo è centrato nell'origine con dimensioni `SceneDims`
//! (larghezza X, altezza Y, profondità Z). La proiezione è prospettica; la
//! telecamera orbita attorno all'oggetto tramite imbardata (`yaw`) e beccheggio
//! (`pitch`), con `zoom` che regola la scala.
//!
//! Il modulo è indipendente da egui: restituisce coordinate di schermo come
//! coppie `(x, y)` con `y` verso il basso, così da poter essere riusato sia
//! dalla vista interattiva sia dall'esportazione PDF.

use crate::models::{SceneDims, SensorPos};

/// Vettore 3D essenziale.
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

/// Parametri della telecamera orbitale.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Rotazione attorno all'asse verticale, in radianti.
    pub yaw: f32,
    /// Inclinazione verticale, in radianti (limitata a ±~85°).
    pub pitch: f32,
    /// Fattore di ingrandimento.
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

/// Numero di facce del parallelepipedo.
pub const FACE_COUNT: usize = 6;

/// Normale uscente della faccia `face` (0..5).
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

/// Distanza dell'origine dal piano che contiene la faccia `face`
/// (l'equazione del piano è `normale · p = offset`).
fn face_plane_offset(face: usize, dims: SceneDims) -> f32 {
    match face {
        0 | 1 => dims.w * 0.5,
        2 | 3 => dims.h * 0.5,
        _ => dims.d * 0.5,
    }
}

/// Punto locale sulla faccia `face` per coordinate `u`, `v` in `[0, 1]`.
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

/// Inversa di [`face_point`]: coordinate `(u, v)` sulla faccia per un punto locale.
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

/// Proiettore prospettico: converte punti locali in coordinate di schermo.
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
    /// Crea un proiettore con centro `(cx, cy)` e `viewport` = lato utile
    /// dell'area di disegno (in pixel o millimetri: le unità restano coerenti).
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

    /// Applica la rotazione della telecamera (imbardata + beccheggio).
    pub fn rotate(&self, p: Vec3) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let x1 = cy * p.x + sy * p.z;
        let z1 = -sy * p.x + cy * p.z;
        let (sp, cp) = self.pitch.sin_cos();
        let y2 = cp * p.y - sp * z1;
        let z2 = sp * p.y + cp * z1;
        Vec3::new(x1, y2, z2)
    }

    /// Inversa della rotazione della telecamera.
    pub fn rotate_inv(&self, q: Vec3) -> Vec3 {
        let (sp, cp) = self.pitch.sin_cos();
        let y1 = cp * q.y + sp * q.z;
        let z1 = -sp * q.y + cp * q.z;
        let (sy, cy) = self.yaw.sin_cos();
        let x = cy * q.x - sy * z1;
        let z = sy * q.x + cy * z1;
        Vec3::new(x, y1, z)
    }

    /// Proietta un punto locale in coordinate di schermo `(x, y)` con `y` in basso.
    pub fn project(&self, p: Vec3) -> (f32, f32) {
        let q = self.rotate(p);
        let dz = (self.dist - q.z).max(1e-4);
        (self.cx + self.focal * q.x / dz, self.cy - self.focal * q.y / dz)
    }

    /// Distanza dalla telecamera (valori minori = più vicini).
    pub fn depth(&self, p: Vec3) -> f32 {
        self.dist - self.rotate(p).z
    }

    /// Punto locale che si proietta esattamente su `(sx, sy)` **mantenendo la
    /// stessa distanza dalla telecamera** di `p`.
    ///
    /// È il trascinamento libero: il punto scorre nel piano parallelo alla
    /// visuale che passa per `p` e non viene riportato su nessuna parete, perciò
    /// può staccarsi dal container o entrarvi. Poiché quel piano è obliquo
    /// rispetto al box ogni volta che la telecamera guarda da tre quarti, il
    /// movimento che ne risulta ha componenti su `x`, `y` e `z` insieme: mentre
    /// si trascina si sfruttano tutti e tre gli assi, non i soli due della
    /// parete.
    ///
    /// La profondità è quella iniziale e resta fissa, quindi il punto non si
    /// avvicina né si allontana mentre si muove.
    ///
    /// Se la telecamera guarda esattamente lungo gli assi il piano di
    /// trascinamento coincide con un piano coordinato e una sola dimensione
    /// resta invariata: è la geometria, non un limite della funzione.
    pub fn drag_at_depth(&self, p: Vec3, sx: f32, sy: f32) -> Vec3 {
        let q = self.rotate(p);
        let dz = (self.dist - q.z).max(1e-4);
        let qx = (sx - self.cx) * dz / self.focal;
        let qy = -(sy - self.cy) * dz / self.focal;
        self.rotate_inv(Vec3::new(qx, qy, q.z))
    }

    /// Ricostruisce il raggio (origine, direzione) in coordinate locali a partire
    /// da un punto di schermo.
    pub fn unproject(&self, sx: f32, sy: f32) -> (Vec3, Vec3) {
        let ax = (sx - self.cx) / self.focal;
        let ay = -(sy - self.cy) / self.focal;
        let origin = self.rotate_inv(Vec3::new(0.0, 0.0, self.dist));
        let dir = self.rotate_inv(Vec3::new(ax, ay, -1.0)).normalized();
        (origin, dir)
    }
}

/// Una faccia proiettata, pronta per il disegno.
#[derive(Clone, Copy)]
pub struct FaceQuad {
    /// I quattro angoli proiettati, nell'ordine (0,0) (1,0) (1,1) (0,1).
    pub pts: [(f32, f32); 4],
    /// Profondità media (per l'ordinamento dal fondo verso la telecamera).
    pub depth: f32,
    /// Fattore di luce 0.45..1.0 calcolato dalla normale della faccia.
    pub shade: f32,
}

/// Proietta le sei facce e le ordina da quella più lontana a quella più vicina.
pub fn face_quads(proj: &Projector, dims: SceneDims) -> Vec<FaceQuad> {
    // Luce fissa nello spazio della telecamera (dall'alto a sinistra, verso
    // l'osservatore) così che le facce frontali risultino illuminate.
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

/// Individua la faccia colpita dal raggio `origin + t·dir` e restituisce
/// `(faccia, u, v)`. Viene scelta la faccia più vicina alla telecamera.
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

/// Posizione 3D (locale) di un sensore salvato.
///
/// Un sensore mai trascinato giace sulla parete `face` alle coordinate `u`, `v`
/// e segue il container quando ne cambiano le dimensioni. Uno trascinato è
/// conservato in coordinate normalizzate (frazi della mezza dimensione) e viene
/// riportato in metri moltiplicando per le mezze dimensioni correnti: anche i
/// sensori staccati dalla parete si adattano quindi agli nuovi ingombri.
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

/// Converte una posizione locale assoluta in coordinate normalizzate, cioè
/// l'operazione inversa di [`sensor_point`] per i sensori trascinati.
///
/// Restituisce frazi della mezza dimensione: la faccia `x = larghezza/2`
/// corrisponde a `x = 1.0`, l'origine a `0.0`.
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
        // Raggio lungo -Z: colpisce la faccia +Z (indice 4).
        let origin = Vec3::new(0.5, 0.0, 10.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        let (face, u, v) = pick_face(origin, dir, d).expect("il raggio colpisce il solido");
        assert_eq!(face, 4);
        // u lungo +X: (0.5 + 3.0) / 6.0 = 0.5833…
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

    /// Il trascinamento libero porta il punto esattamente sotto il cursore e non
    /// ne cambia la distanza dalla telecamera. Non c'è alcun vincolo di parete,
    /// quindi vale anche quando il punto è già staccato dal container.
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

                        // Cade esattamente dove si punta...
                        let (ax, ay) = proj.project(p);
                        let scarto = ((ax - sx - dx).powi(2) + (ay - sy - dy).powi(2)).sqrt();
                        assert!(
                            scarto < 0.5,
                            "yaw={yaw} pitch={pitch} faccia={face} delta=({dx}, {dy}): \
                             il punto cade a {scarto:.2}px dal cursore"
                        );

                        // ...senza avvicinarsi né allontanarsi dalla camera.
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

    /// Con la telecamera da tre quarti il piano di trascinamento è obliquo
    /// rispetto al container: muovendo il cursore il punto si sposta con
    /// componenti su tutte e tre le direzioni, non sui soli due assi della
    /// parete su cui giaceva.
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

        // Due direzioni ortogonali sullo schermo generano il piano di movimento.
        let direzioni = [
            proj.drag_at_depth(p0, sx + 60.0, sy).add(p0.scale(-1.0)),
            proj.drag_at_depth(p0, sx, sy + 60.0).add(p0.scale(-1.0)),
        ];
        let raggio = direzioni
            .iter()
            .fold(0.0_f32, |a, v| a.max((v.x * v.x + v.y * v.y + v.z * v.z).sqrt()));
        assert!(raggio > 0.5, "spostamento troppo piccolo: {raggio}");

        // Nessuna delle tre direzioni deve restare fuori portata.
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
