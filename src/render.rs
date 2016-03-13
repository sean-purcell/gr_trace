extern crate glium;
extern crate clap; 
extern crate image;
extern crate cgmath;

use glium::backend::glutin_backend::GlutinFacade;
use clap::ArgMatches;
use std::f32;

struct RenderBuffers(glium::VertexBuffer<RayVertex>, glium::IndexBuffer<u8>);
pub struct Renderer {
    program: glium::Program,
    background: glium::texture::Texture2d,

    buffers: RenderBuffers,

    t: f32,
}

impl Renderer {
    pub fn new(display: &GlutinFacade, args: &ArgMatches) -> Self {
        let bg = {
            use std::io::Cursor;
            //FIXME: insert alternate bg images here

            let im = image::load(
                        Cursor::new(&include_bytes!("../resources/bg.jpg")[..]),
                        image::JPEG).unwrap().to_rgba();

            let imdim = im.dimensions();
            let im = glium::texture::RawImage2d::from_raw_rgba_reversed(
                        im.into_raw(), imdim);
            glium::texture::Texture2d::new(display, im).unwrap()
        };

        let prog = {
            //FIXME: add option for different shaders
            let shaders = &FLAT_SHADER;

            glium::Program::from_source(display,
                shaders.vert_shader,
                shaders.frag_shader,
                None).unwrap() 
        };

        let bufs = {
            use glium::index::PrimitiveType::TrianglesList;
            RenderBuffers(glium::VertexBuffer::new(display, &VERTICES).unwrap(),
             glium::IndexBuffer::new(display, TrianglesList, &INDICES).unwrap())
        };

        Renderer { program: prog, background: bg, buffers: bufs, t: 0.0 }
    }

    pub fn render(&self, display: &GlutinFacade) {
        use glium::Surface;

        let mut target = display.draw();

        target.clear_color(0., 0., 0., 1.0);

        let (width, height) = target.get_dimensions();

        let facing_mat = {
            use cgmath::*;
            let src = Point3::new(0.0f32,0.,0.);
            let tow = Point3::new(self.t.sin(), 0.0f32, self.t.cos());
            let up = vec3(0.,1.,0.0f32);

            Into::<[[f32;4];4]>::into(cgmath::Matrix4::look_at(src, tow, up))
        };


        let uniforms = uniform! {
            height_ratio: (height as f32) / (width as f32),
            fov_ratio: (f32::consts::FRAC_PI_2/2.).tan(), // pi/2, 90 deg
            facing: facing_mat,
            tex: &self.background,
        };

        target.draw(&self.buffers.0, &self.buffers.1, &self.program,
                    &uniforms,
                    &Default::default()).unwrap();

        target.finish().unwrap();
    }
}

#[derive(Copy, Clone)]
struct RayVertex {
    pos: (f32, f32),
}

implement_vertex!(RayVertex, pos);

const VERTICES: [RayVertex; 4] = [
    RayVertex { pos: (-1., -1.) },
    RayVertex { pos: (-1.,  1.) },
    RayVertex { pos: ( 1.,  1.) },
    RayVertex { pos: ( 1., -1.) },
];

const INDICES: [u8; 6] = [
    0, 1, 2,
    0, 2, 3
];

struct ShaderPair {
    vert_shader: &'static str,
    frag_shader: &'static str,
}

const FLAT_SHADER: ShaderPair = ShaderPair {
    vert_shader: r#"

#version 140

in vec2 pos;
out vec3 dir;
out vec2 pos_v;

uniform float height_ratio; // height / width
uniform float fov_ratio; // tan(fov / 2)

uniform mat4 facing;

void main() {
    float x = pos.x * fov_ratio;
    float y = pos.y * fov_ratio * height_ratio;
    dir = normalize(vec3(facing * vec4(x, y, 1.0, 1.0)));
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

    "#,
    frag_shader: r#"

#version 140

in vec3 dir;
in vec2 pos_v;

out vec4 color;

uniform sampler2D tex;

#define M_PI 3.1415926535897932384626433832795

float yaw(vec3 v) {
    if(abs(v.y) >= 0.999) {
        return 0;
    }
    return atan(v.x, v.z);
}

float pitch(vec3 v) {
    return asin(v.y);
}

float as_coord(float ang) {
    return (ang + M_PI / 2.) / M_PI;
}

void main() {
    vec2 tex_coords = vec2(as_coord(yaw(dir)), as_coord(pitch(dir)));

    color = texture(tex, tex_coords);
}

    "#,
};

const GR_SHADER: ShaderPair = ShaderPair {
    vert_shader: r#"

#version 140

in vec2 pos;
out vec2 pos_v;

void main() {
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

    "#,
    frag_shader: r#"

#version 140

in vec2 pos_v;

out vec4 color;

void main() {
    float red = (pos_v.x + 1) / 2.;
    float green = (pos_v.y + 1) / 2.;
    color = vec4(red, green, 0.0, 1.0);
}

    "#,
};
