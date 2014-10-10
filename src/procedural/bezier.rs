use std::ptr;
use na;
use na::{Cast, FloatPnt, FloatPntExt, FloatVec, FloatVecExt};
use procedural::{TriMesh, Polyline};
use procedural;

// De-Casteljau algorithm.
// Evaluates the bezier curve with control points `control_points`.
#[doc(hidden)]
pub fn bezier_curve_at<N: Float + Clone + Cast<f64>,
                       P: Clone + FloatPnt<N, V>,
                       V: FloatVec<N>>(
                       control_points: &[P],
                       t:              &N,
                       cache:          &mut Vec<P>)
                       -> P {
    if control_points.len() > cache.len() {
        let diff = control_points.len() - cache.len(); 
        cache.grow(diff, na::orig())
    }

    let cache = cache.as_mut_slice();

    let _1: N = na::cast(1.0);
    let t_1   = _1 - *t;

    // XXX: not good if the objects are not POD.
    unsafe {
        ptr::copy_memory(cache.as_mut_ptr(), control_points.as_ptr(), control_points.len());
    }

    for i in range(1u, control_points.len()) {
        for j in range(0u, control_points.len() - i) {
            cache[j] = cache[j] * t_1 + *cache[j + 1].as_vec() * *t;
        }
    }

    cache[0].clone()
}

// Evaluates the bezier curve with control points `control_points`.
#[doc(hidden)]
pub fn bezier_surface_at<N: Float + Clone + Cast<f64>,
                         P: Clone + FloatPnt<N, V>,
                         V: FloatVec<N>>(
                         control_points: &[P],
                         nupoints:       uint,
                         nvpoints:       uint,
                         u:              &N,
                         v:              &N,
                         ucache:         &mut Vec<P>,
                         vcache:         &mut Vec<P>)
                         -> P {
    if vcache.len() < nvpoints {
        let diff = nvpoints - vcache.len();
        vcache.grow(diff, na::orig());
    }

    // FIXME: start with u or v, depending on which dimension has more control points.
    let vcache = vcache.as_mut_slice();

    for i in range(0, nvpoints) {
        let start = i * nupoints;
        let end   = start + nupoints;

        vcache[i] = bezier_curve_at(control_points.slice(start, end), u, ucache);
    }

    bezier_curve_at(vcache.slice(0, nvpoints), v, ucache)
}

/// Given a set of control points, generates a (non-rational) Bezier curve.
pub fn bezier_curve<N: Float + Clone + Cast<f64>,
                    P: Clone + FloatPnt<N, V>,
                    V: Clone + FloatVec<N>>(
                    control_points: &[P],
                    nsubdivs:       uint)
                    -> Polyline<N, P, V> {
    let mut coords = Vec::with_capacity(nsubdivs);
    let mut cache  = Vec::new();
    let tstep      = na::cast(1.0 / (nsubdivs as f64));
    let mut t      = na::zero::<N>();

    while t <= na::one() {
        coords.push(bezier_curve_at(control_points, &t, &mut cache));
        t = t + tstep;
    }

    // FIXME: normals

    Polyline::new(coords, None)
}

/// Given a set of control points, generates a (non-rational) Bezier surface.
pub fn bezier_surface<N: Float + Clone + Cast<f64>,
                      P: Clone + FloatPntExt<N, V>,
                      V: FloatVecExt<N>>(
                      control_points: &[P],
                      nupoints:       uint,
                      nvpoints:       uint,
                      usubdivs:       uint,
                      vsubdivs:       uint)
                      -> TriMesh<N, P, V> {
    assert!(nupoints * nvpoints == control_points.len());

    let mut surface = procedural::unit_quad(usubdivs, vsubdivs);

    {
        let uvs    = surface.uvs.as_ref().unwrap().as_slice();
        let coords = surface.coords.as_mut_slice();

        let mut ucache = Vec::new();
        let mut vcache = Vec::new();

        for j in range(0, vsubdivs + 1) {
            for i in range(0, usubdivs + 1) {
                let id = i + j * (usubdivs + 1);
                coords[id] = bezier_surface_at(control_points,
                                               nupoints,
                                               nvpoints,
                                               &uvs[id].x,
                                               &uvs[id].y,
                                               &mut ucache,
                                               &mut vcache)
            }
        }

        // XXX: compute the normals manually.
        surface.normals = None;
    }

    surface
}