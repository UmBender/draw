#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn vertex(n: u32) -> Vertex {
        Vertex {
            pos: [n as f32, 0.0],
            uv: [0.0, 0.0],
            color: [1, 2, 3, 4],
        }
    }

    fn vertices(count: usize) -> Vec<Vertex> {
        (0..count as u32).map(vertex).collect()
    }

    // ---- AC-1 chunks ---------------------------------------------------

    #[test]
    fn chunks_of_small_geometry_is_one_chunk() {
        // Arrange
        let v = vertices(3);

        // Act
        let chunks = chunks(&v, &[0, 1, 2]).unwrap();

        // Assert
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].indices, vec![0, 1, 2]);
        assert_eq!(chunks[0].vertices.len(), 3);
    }

    #[test]
    fn chunks_of_empty_geometry_is_empty() {
        assert!(chunks(&[], &[]).unwrap().is_empty());
    }

    #[test]
    fn chunks_split_at_the_index_limit() {
        // Arrange: one shared triangle repeated past the index limit.
        let v = vertices(3);
        let triangles = CHUNK_MAX_INDICES / 3 + 1;
        let indices: Vec<u32> = (0..triangles).flat_map(|_| [0, 1, 2]).collect();

        // Act
        let chunks = chunks(&v, &indices).unwrap();

        // Assert
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].indices.len(), CHUNK_MAX_INDICES);
        assert_eq!(chunks[1].indices.len(), 3);
    }

    #[test]
    fn chunks_split_at_the_vertex_limit() {
        // Arrange: disjoint triangles, three new vertices each.
        let triangles = CHUNK_MAX_VERTICES / 3 + 1;
        let v = vertices(triangles * 3);
        let indices: Vec<u32> = (0..(triangles * 3) as u32).collect();

        // Act
        let chunks = chunks(&v, &indices).unwrap();

        // Assert
        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|c| c.vertices.len() <= CHUNK_MAX_VERTICES));
    }

    #[test]
    fn chunks_remap_indices_to_local_vertices() {
        // Arrange: the second triangle uses vertices 3, 4, 5 only.
        let v = vertices(6);
        let indices = [0, 1, 2, 3, 4, 5];

        // Act
        let one = chunks(&v, &indices).unwrap();
        let second = chunks(&v, &indices[3..]).unwrap();

        // Assert
        assert_eq!(one[0].indices, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(second[0].indices, vec![0, 1, 2]);
        assert_eq!(second[0].vertices[0], v[3]);
    }

    #[test]
    fn chunks_reject_index_out_of_range() {
        let v = vertices(3);
        assert!(matches!(
            chunks(&v, &[0, 1, 3]),
            Err(GpuError::Backend(_))
        ));
    }

    #[test]
    fn chunks_reject_partial_triangle() {
        let v = vertices(3);
        assert!(matches!(chunks(&v, &[0, 1]), Err(GpuError::Backend(_))));
    }

    proptest! {
        #[test]
        fn chunks_cover_every_triangle_within_limits(
            count in 3usize..40,
            picks in proptest::collection::vec(0u32..40, 0..9_000),
        ) {
            // Arrange
            let v = vertices(count);
            let triangles = picks.len() / 3;
            let indices: Vec<u32> = picks[..triangles * 3]
                .iter()
                .map(|i| i % count as u32)
                .collect();

            // Act
            let chunks = chunks(&v, &indices).unwrap();

            // Assert: limits hold and the triangles come back in order.
            let mut back = Vec::new();
            for chunk in &chunks {
                prop_assert!(chunk.vertices.len() <= CHUNK_MAX_VERTICES);
                prop_assert!(chunk.indices.len() <= CHUNK_MAX_INDICES);
                prop_assert_eq!(chunk.indices.len() % 3, 0);
                for &i in &chunk.indices {
                    back.push(chunk.vertices[usize::from(i)]);
                }
            }
            let want: Vec<Vertex> = indices.iter().map(|&i| v[i as usize]).collect();
            prop_assert_eq!(back, want);
        }
    }

    // ---- AC-2 handles --------------------------------------------------

    #[test]
    fn table_get_after_insert_returns_value() {
        let mut table = Table::new();
        let id = table.insert("a");
        assert_eq!(table.get(id).unwrap(), &"a");
    }

    #[test]
    fn table_get_unknown_id_is_unknown_handle() {
        let table: Table<u8> = Table::new();
        assert_eq!(table.get(7).unwrap_err(), GpuError::UnknownHandle);
    }

    #[test]
    fn table_ids_are_unique() {
        let mut table = Table::new();
        let a = table.insert(1);
        let b = table.insert(2);
        assert_ne!(a, b);
    }

    // ---- AC-3 passes ---------------------------------------------------

    fn run(steps: &[Step]) -> Result<(), GpuError> {
        let mut checker = PassChecker::default();
        for &step in steps {
            checker.step(step)?;
        }
        checker.finish()
    }

    #[test]
    fn passes_balanced_list_is_ok() {
        assert!(run(&[Step::Begin, Step::Draw, Step::Draw, Step::End]).is_ok());
    }

    #[test]
    fn passes_draw_outside_pass_is_error() {
        assert!(matches!(run(&[Step::Draw]), Err(GpuError::Backend(_))));
    }

    #[test]
    fn passes_nested_begin_is_error() {
        assert!(matches!(
            run(&[Step::Begin, Step::Begin]),
            Err(GpuError::Backend(_))
        ));
    }

    #[test]
    fn passes_end_without_begin_is_error() {
        assert!(matches!(run(&[Step::End]), Err(GpuError::Backend(_))));
    }

    #[test]
    fn passes_unfinished_list_is_error() {
        assert!(matches!(run(&[Step::Begin]), Err(GpuError::Backend(_))));
    }
}
