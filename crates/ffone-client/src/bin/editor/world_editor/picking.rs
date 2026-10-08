//! Picking uses displayed mesh bounds, including large models away from their pivot.
use super::*;
use bevy::camera::primitives::Aabb;

fn distance(ray: Ray3d, bounds: &Aabb, transform: &GlobalTransform) -> Option<f32> {
    let inverse = transform.to_matrix().inverse();
    let origin = inverse.transform_point3(ray.origin);
    let direction = inverse.transform_vector3(*ray.direction);
    let low = Vec3::from(bounds.center - bounds.half_extents);
    let high = Vec3::from(bounds.center + bounds.half_extents);
    let (mut near, mut far) = (0f32, f32::INFINITY);
    for axis in 0..3 {
        if direction[axis].abs() < 1e-8 {
            if origin[axis] < low[axis] || origin[axis] > high[axis] {
                return None;
            }
            continue;
        }
        let a = (low[axis] - origin[axis]) / direction[axis];
        let b = (high[axis] - origin[axis]) / direction[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if far < near {
            return None;
        }
    }
    (near.is_finite() && far >= 0.).then_some(near)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transformed_mesh_bounds_use_world_ray_distance_and_reject_parallel_misses() {
        let bounds = Aabb::from_min_max(Vec3::splat(-1.), Vec3::splat(1.));
        let transform = GlobalTransform::from(
            Transform::from_xyz(0., 0., 10.)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(2., 3., 4.)),
        );
        let ray = Ray3d::new(Vec3::ZERO, Dir3::Z);
        assert!((distance(ray, &bounds, &transform).unwrap() - 8.).abs() < 0.001);
        assert!(
            distance(
                Ray3d::new(Vec3::new(5., 0., 0.), Dir3::Z),
                &bounds,
                &transform
            )
            .is_none()
        );
    }
}
pub(super) fn entity(
    e: &WorldEditor,
    preview: &preview::WorldPreview,
    ray: Ray3d,
    meshes: &Query<(Entity, &Aabb, &GlobalTransform)>,
    parents: &Query<&ChildOf>,
) -> Option<usize> {
    let owners: std::collections::HashMap<_, _> = preview
        .objects
        .iter()
        .map(|(key, entity)| (*entity, key))
        .collect();
    // Every rendered object can be picked in object mode, independently of the
    // keyword used to reduce the list and 2D markers.
    let candidates = e.pickable();
    meshes
        .iter()
        .filter_map(|(entity, bounds, transform)| {
            let t = distance(ray, bounds, transform)?;
            let mut current = entity;
            let mut key = None;
            for _ in 0..48 {
                if let Some(owner) = owners.get(&current) {
                    key = Some(owner.as_str());
                    break;
                }
                current = parents.get(current).ok()?.parent();
            }
            let key = key?;
            let index = candidates.iter().copied().find(|i| {
                let p = &e.entities[*i];
                if p.kind != 4 {
                    key == p.key
                } else {
                    let Some(node) = e.sources[p.source]
                        .draft
                        .pointer(&p.pointer)
                        .and_then(|v| v["sourceNode"].as_str())
                    else {
                        return false;
                    };
                    key.split_once(":visual:").or_else(||key.split_once(":collision:")).is_some_and(|(tile, name)| {
                        p.key.starts_with(&format!("{tile}/")) && map::matches_node(name, node)
                    })
                }
            })?;
            Some((index, t))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

pub(super) fn focus(e:&mut WorldEditor,preview:&preview::WorldPreview,meshes:&Query<(Entity,&Aabb,&GlobalTransform)>,parents:&Query<&ChildOf>){
    let Some(p)=e.selected().filter(|p|p.kind==4)else{return;};
    let Some(node)=e.sources[p.source].draft.pointer(&p.pointer).and_then(|r|r["sourceNode"].as_str())else{return;};
    let roots:BTreeSet<_>=preview.objects.iter().filter_map(|(key,entity)|{
        let (tile,name)=key.split_once(":visual:").or_else(||key.split_once(":collision:"))?;
        (p.key.starts_with(&format!("{tile}/"))&&map::matches_node(name,node)).then_some(*entity)
    }).collect();
    let mut low=Vec3::splat(f32::INFINITY);let mut high=Vec3::splat(f32::NEG_INFINITY);
    for (entity,bounds,transform) in meshes {
        let mut current=entity;let mut owned=false;
        for _ in 0..48{if roots.contains(&current){owned=true;break;}let Ok(parent)=parents.get(current)else{break;};current=parent.parent();}
        if !owned{continue;}
        let center=Vec3::from(bounds.center);let half=Vec3::from(bounds.half_extents);
        for x in [-1.,1.]{for y in [-1.,1.]{for z in [-1.,1.]{let corner=transform.transform_point(center+half*Vec3::new(x,y,z));low=low.min(corner);high=high.max(corner);}}}
    }
    if low.is_finite()&&high.is_finite(){e.center=(low+high)*0.5;e.distance=((high-low).length()*1.5+5.).clamp(40.,2000.);e.revision+=1;}
}
