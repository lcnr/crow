use glium::{Display, VertexBuffer};

use crate::ErrDontCare;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

glium::implement_vertex!(Vertex, position);

pub fn initialize_vertex_buffer(display: &Display) -> Result<VertexBuffer<Vertex>, ErrDontCare> {
    let vertex1 = Vertex {
        position: [0.0, 0.0],
    };
    let vertex2 = Vertex {
        position: [1.0, 0.0],
    };
    let vertex3 = Vertex {
        position: [1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [0.0, 1.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    VertexBuffer::new(display, &shape).map_err(|err| {
        todo!("initialize_vertex_buffer: {:?}", err);
        ErrDontCare
    })
}
