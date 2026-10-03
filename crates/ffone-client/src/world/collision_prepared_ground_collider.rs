use super::*;

pub(super) struct PreparedGroundCollider {
    pub(super) vertices: Arc<[Vec3]>,
    pub(super) indices: Arc<[u32]>,
    pub(super) world_from_local: Mat4,
    pub(super) triangles: Vec<PreparedGroundTriangle>,
    pub(super) minimum: Vec2,
    pub(super) maximum: Vec2,
    pub(super) cells: Vec<GroundTriangleCell>,
    pub(super) triangle_cursor: usize,
    pub(super) cell_cursor: usize,
    pub(super) cells_initialized: bool,
    pub(super) ready: bool,
}

pub(super) const GROUND_TRIANGLE_GRID_SIDE: usize = 8;

impl PreparedGroundCollider {
    pub(super) fn start(
        collider: &AuthoredTriMeshCollider,
        matrix: Mat4,
        bounds: &AuthoredColliderWorldBounds,
    ) -> Self {
        Self {
            vertices: collider.vertices.clone(),
            indices: collider.indices.clone(),
            world_from_local: matrix,
            triangles: Vec::new(),
            minimum: Vec2::new(bounds.minimum.x, bounds.minimum.z),
            maximum: Vec2::new(bounds.maximum.x, bounds.maximum.z),
            cells: Vec::new(),
            triangle_cursor: 0,
            cell_cursor: 0,
            cells_initialized: false,
            ready: false,
        }
    }

    #[cfg(test)]
    pub(super) fn new(
        collider: &AuthoredTriMeshCollider,
        matrix: Mat4,
        bounds: &AuthoredColliderWorldBounds,
    ) -> Self {
        let mut prepared = Self::start(collider, matrix, bounds);
        prepared.advance(&mut GroundPreparationBudget {
            triangles: usize::MAX,
            cell_tests: usize::MAX,
        });
        prepared
    }

    pub(super) fn advance(&mut self, budget: &mut GroundPreparationBudget) {
        while self.triangle_cursor < self.indices.len() / 3 {
            if budget.triangles == 0 {
                return;
            }
            budget.triangles -= 1;
            let triangle = &self.indices[self.triangle_cursor * 3..][..3];
            self.triangle_cursor += 1;
            let a = self
                .world_from_local
                .transform_point3(self.vertices[triangle[0] as usize]);
            let b = self
                .world_from_local
                .transform_point3(self.vertices[triangle[1] as usize]);
            let c = self
                .world_from_local
                .transform_point3(self.vertices[triangle[2] as usize]);
            let normal = (b - a).cross(c - a).normalize_or_zero();
            if normal.is_finite()
                && normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT
                && a.is_finite()
                && b.is_finite()
                && c.is_finite()
            {
                self.triangles.push(PreparedGroundTriangle { a, b, c });
            }
        }
        if !self.cells_initialized {
            self.cells_initialized = true;
            let extent = self.maximum - self.minimum;
            if self.triangles.len() >= 32
                && self.minimum.is_finite()
                && self.maximum.is_finite()
                && extent.is_finite()
                && extent.cmpgt(Vec2::ZERO).all()
            {
                for z in 0..GROUND_TRIANGLE_GRID_SIDE {
                    for x in 0..GROUND_TRIANGLE_GRID_SIDE {
                        let minimum = self.minimum
                            + extent * Vec2::new(x as f32, z as f32)
                                / GROUND_TRIANGLE_GRID_SIDE as f32;
                        let maximum = self.minimum
                            + extent * Vec2::new((x + 1) as f32, (z + 1) as f32)
                                / GROUND_TRIANGLE_GRID_SIDE as f32;
                        self.cells.push(GroundTriangleCell {
                            minimum,
                            maximum,
                            triangles: Vec::new(),
                        });
                    }
                }
            }
        }
        while self.cell_cursor < self.cells.len() * self.triangles.len() {
            if budget.cell_tests == 0 {
                return;
            }
            budget.cell_tests -= 1;
            let cell = &mut self.cells[self.cell_cursor / self.triangles.len()];
            let index = self.cell_cursor % self.triangles.len();
            self.cell_cursor += 1;
            if self.triangles[index].may_hit_cell(cell.minimum, cell.maximum) {
                cell.triangles.push(index);
            }
        }
        self.ready = true;
    }

    pub(super) fn matches(&self, collider: &AuthoredTriMeshCollider, matrix: Mat4) -> bool {
        // Identity only authorizes reuse of the very same immutable geometry,
        // not merging different colliders. Replaced geometry always rebuilds.
        Arc::ptr_eq(&self.vertices, &collider.vertices)
            && Arc::ptr_eq(&self.indices, &collider.indices)
            && self.world_from_local.to_cols_array().map(f32::to_bits)
                == matrix.to_cols_array().map(f32::to_bits)
    }

    pub(super) fn candidates(&self, x: f32, z: f32) -> Option<&[usize]> {
        if self.cells.is_empty() {
            return None;
        }
        let point = Vec2::new(x, z);
        if !point.is_finite()
            || !point.cmpge(self.minimum).all()
            || !point.cmple(self.maximum).all()
        {
            return None;
        }
        let coordinate = (point - self.minimum) / (self.maximum - self.minimum)
            * GROUND_TRIANGLE_GRID_SIDE as f32;
        let cell = &self.cells[(coordinate.y as usize).min(GROUND_TRIANGLE_GRID_SIDE - 1)
            * GROUND_TRIANGLE_GRID_SIDE
            + (coordinate.x as usize).min(GROUND_TRIANGLE_GRID_SIDE - 1)];
        // Division/rounding at a cell boundary may select its neighbor. Use the
        // index only when its evaluated interval actually contains this point.
        (point.cmpge(cell.minimum).all() && point.cmple(cell.maximum).all())
            .then_some(cell.triangles.as_slice())
    }

    pub(super) fn height(&self, x: f32, z: f32, minimum_y: f32, maximum_y: f32) -> Option<f32> {
        let mut highest = None;
        let mut visit = |triangle: &PreparedGroundTriangle| {
            let Some(height) = triangle_height_at_xz(triangle.a, triangle.b, triangle.c, x, z)
            else {
                return;
            };
            if height < minimum_y - GROUND_EPSILON || height > maximum_y + GROUND_EPSILON {
                return;
            }
            if highest.is_none_or(|current| height > current) {
                highest = Some(height);
            }
        };
        if let Some(indices) = self.candidates(x, z) {
            for index in indices {
                visit(&self.triangles[*index]);
            }
        } else {
            for triangle in &self.triangles {
                visit(triangle);
            }
        }
        highest
    }
}

pub(super) struct GroundTriangleCache {
    pub(super) entries: HashMap<Entity, PreparedGroundCollider>,
    pub(super) used: HashSet<Entity>,
    pub(super) enabled: bool,
    pub(super) budget: GroundPreparationBudget,
}

impl Default for GroundTriangleCache {
    fn default() -> Self {
        Self {
            entries: default(),
            budget: default(),
            used: default(),
            enabled: !(std::env::var_os("FFONE_PERF_OUTPUT").is_some()
                && std::env::var_os("FFONE_PERF_CPU_QUERY_BASELINE").is_some()),
        }
    }
}

impl GroundTriangleCache {
    pub(super) fn begin_frame(&mut self, mut exists: impl FnMut(Entity) -> bool) {
        self.entries
            .retain(|entity, _| self.used.contains(entity) && exists(*entity));
        self.used.clear();
        self.budget = GroundPreparationBudget::default();
    }

    pub(super) fn height(
        &mut self,
        entity: Entity,
        collider: &AuthoredTriMeshCollider,
        matrix: Mat4,
        bounds: &AuthoredColliderWorldBounds,
        x: f32,
        z: f32,
        minimum_y: f32,
        maximum_y: f32,
    ) -> Option<f32> {
        if !self.enabled {
            return collider_ground_height_with_bounds(
                collider, matrix, bounds, x, z, minimum_y, maximum_y,
            );
        }
        let epsilon = Vec3::splat(GROUND_EPSILON);
        if !bounds.overlaps(
            Vec3::new(x, minimum_y, z) - epsilon,
            Vec3::new(x, maximum_y, z) + epsilon,
        ) {
            return None;
        }
        self.used.insert(entity);
        let prepared = self
            .entries
            .entry(entity)
            .or_insert_with(|| PreparedGroundCollider::start(collider, matrix, bounds));
        if !prepared.matches(collider, matrix) {
            *prepared = PreparedGroundCollider::start(collider, matrix, bounds);
        }
        if !prepared.ready {
            prepared.advance(&mut self.budget);
        }
        if prepared.ready {
            prepared.height(x, z, minimum_y, maximum_y)
        } else {
            collider_ground_height_with_bounds(collider, matrix, bounds, x, z, minimum_y, maximum_y)
        }
    }
}
