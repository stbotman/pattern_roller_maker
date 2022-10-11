#[cfg(debug_assertions)]
use crate::vectors::check_right_hand;
use crate::vectors::Vector3;
use crate::Parameters;
use anyhow::{Context, Error, Result};
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct STLFileWriter {
    buffered_file: BufWriter<File>,
    faces_count: u32,
}

impl STLFileWriter {
    const HEADER_SIZE: usize = 80;
    const HEADER_TEXT: [u8; 14] = *b"pattern roller";
    const SPACER: [u8; 2] = [0u8; 2];

    fn write_data(&mut self, data: &[u8]) -> Result<()> {
        self.buffered_file.write_all(data).map_err(Error::from)
    }

    fn write_header(&mut self) -> Result<()> {
        let mut header: Vec<u8> = Vec::with_capacity(STLFileWriter::HEADER_SIZE);
        header.extend_from_slice(&STLFileWriter::HEADER_TEXT);
        header.resize(STLFileWriter::HEADER_SIZE, 0u8);
        self.write_data(&header)
    }

    fn write_n_faces(&mut self) -> Result<()> {
        self.write_data(&self.faces_count.to_le_bytes())
    }

    pub fn write_face(
        &mut self,
        vec_n: &Vector3,
        vec_a: &Vector3,
        vec_b: &Vector3,
        vec_c: &Vector3,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            debug_face_data(vec_n, vec_a, vec_b, vec_c);
            self.faces_count -= 1;
        }
        self.write_data(&vec_n.to_binary())?;
        self.write_data(&vec_a.to_binary())?;
        self.write_data(&vec_b.to_binary())?;
        self.write_data(&vec_c.to_binary())?;
        self.write_data(&STLFileWriter::SPACER)
    }

    pub fn write_face_auto_normal(
        &mut self,
        vec_a: &Vector3,
        vec_b: &Vector3,
        vec_c: &Vector3,
    ) -> Result<()> {
        let vec_n = face_normal(vec_a, vec_b, vec_c);
        self.write_face(&vec_n, vec_a, vec_b, vec_c)
    }

    pub fn new(params: &Parameters) -> Result<STLFileWriter> {
        let filename = params.output_filename.as_str();
        let file = File::create(filename)
            .with_context(|| format!("Failed to open file '{}' for writing", filename))?;
        let buffered_file = BufWriter::new(file);
        let n_faces = params.faces_count()?;
        let mut stl_writer = STLFileWriter {
            buffered_file: buffered_file,
            faces_count: n_faces,
        };
        stl_writer.write_header()?;
        stl_writer.write_n_faces()?;
        Ok(stl_writer)
    }
}

fn face_normal(vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) -> Vector3 {
    let vec_ab = Vector3::from_points(vec_a, vec_b);
    let vec_ac = Vector3::from_points(vec_a, vec_c);
    let vec_normal = Vector3::from_cross_product(vec_ab, vec_ac).normalize();
    vec_normal
}

#[cfg(debug_assertions)]
impl Drop for STLFileWriter {
    fn drop(&mut self) {
        assert!(
            self.faces_count == 0,
            "Faces count mismatch: stl file was not fully written"
        );
    }
}

#[cfg(debug_assertions)]
fn debug_face_data(vec_n: &Vector3, vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) {
    assert!(
        vec_a != vec_b && vec_b != vec_c && vec_c != vec_a && vec_n != &Vector3::ZERO,
        "Encountered degenerate face: a:{:?} b:{:?} c:{:?} n:{:?}",
        &vec_a,
        &vec_b,
        &vec_c,
        &vec_n
    );
    assert!(
        check_right_hand(
            &Vector3::from_points(vec_b, vec_c),
            &Vector3::from_points(vec_b, vec_a),
            &vec_n
        ),
        "Encountered inverted normal: a:{:?} b:{:?} c:{:?} n:{:?}",
        &vec_a,
        &vec_b,
        &vec_c,
        &vec_n
    );
}
