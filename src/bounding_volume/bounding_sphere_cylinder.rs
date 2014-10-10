use na::Translation;
use math::Matrix;
use bounding_volume::{HasBoundingSphere, BoundingSphere};
use geom::Cone;


impl HasBoundingSphere for Cone {
    #[inline]
    fn bounding_sphere(&self, m: &Matrix) -> BoundingSphere {
        let center = m.translation().to_pnt();
        let radius = (self.radius() * self.radius() + self.half_height() * self.half_height()).sqrt();

        BoundingSphere::new(center, radius)
    }
}