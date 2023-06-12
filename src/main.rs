use std::collections::HashMap;
use std::ffi::c_void;
use egui::{ClippedPrimitive, emath, epaint, ImageData, PlatformOutput, Pos2, Stroke, Style};
use egui::epaint::{Primitive, Shadow};
use glam::{Mat4, Quat, Vec2, Vec3};
use glutin::config::Api;
use glutin::context::GlProfile;
use glutin::surface::Rect;
use stereokit::{Color128, Color32, CullMode, DepthTest, Material, Mesh, RenderLayer, Settings, Shader, SkDraw, StereoKitDraw, StereoKitMultiThread, Tex, TextureFormat, TextureType, Transparency, Vert};
use stereokit_sys::mesh_create;

#[test]
fn test() {
    main();
}

pub fn main() {

    let mut ctx = egui::Context::default();

    let sk = Settings::default().init().unwrap();
    let mut meshes = vec![];
    let mut second_meshes = vec![];
    let mut sk_egui = SkEgui {
        textures: Default::default(),
    };

    sk.run(|sk| {
        let raw_input: egui::RawInput = gather_input();
        let full_output = ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.label("Hello world!");
                if ui.button("Click me").clicked() {

                }
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                if ui.button("YO").clicked() {

                }
            });
        });
        handle_platform_output(full_output.platform_output);
        let clipped_primitives = ctx.tessellate(full_output.shapes); // create triangles to paint
        sk_egui.paint(full_output.textures_delta, clipped_primitives, sk, &mut meshes, &mut second_meshes);
    }, |sk| {});
}

pub fn gather_input() -> egui::RawInput {
    let mut raw_input = egui::RawInput::default();
    raw_input.pixels_per_point = Some(4.0);
    raw_input.screen_rect = Some(egui::Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)));
    raw_input
}

pub fn handle_platform_output(output: PlatformOutput) {

}
fn flip_elements(vec: &mut Vec<u32>) {
    let len = vec.len();
    for i in (0..len).step_by(3) {
        if i + 2 < len {
            vec.swap(i, i + 2);
        }
    }
}
pub struct SkEgui {
    textures: HashMap<egui::TextureId, (Tex, Material)>,
}
impl SkEgui {
    pub fn set_texture(&mut self, tex_id: egui::TextureId, delta: &mut egui::epaint::ImageDelta, sk: &SkDraw) {
        let (texture, _) = self.textures.entry(tex_id).or_insert_with(|| {
            (sk.tex_create(TextureType::DYNAMIC, TextureFormat::RGBA32Linear), sk.material_copy(Material::UNLIT_CLIP))
        });

        match &mut delta.image {
            ImageData::Color(image) => {
                panic!();
                let data: &mut [u8] = bytemuck::cast_slice_mut(image.pixels.as_mut());
                unsafe {
                    stereokit_sys::tex_set_colors(texture.0.as_ptr(), image.size[0] as i32, image.size[1] as i32, data.as_mut_ptr() as *mut c_void);
                }
            }
            ImageData::Font(image) => {
                let mut data: Vec<Color32> = image
                    .srgba_pixels(None)
                    .into_iter().map(|a| Color32::new(a.r(), a.g(), a.b(), a.a()))
                    .collect();
                unsafe {
                    stereokit_sys::tex_set_colors(texture.0.as_ptr(), image.size[0] as i32, image.size[1] as i32, data.as_mut_ptr() as *mut c_void);
                }
            }
        }
    }
    pub fn paint(&mut self, texture: epaint::textures::TexturesDelta, mut primitives: Vec<ClippedPrimitive>, sk: &SkDraw, x: &mut Vec<Mesh>, y: &mut Vec<Mesh>) {
        let pos = Mat4::from_scale_rotation_translation([0.01, 0.01, 0.01].into(), Quat::IDENTITY, Vec3::default());
        for (id, mut image_delta) in texture.set {
            self.set_texture(id, &mut image_delta, sk);
        }
        let mut pos = Vec3::new(0.0, 0.0, 0.0);
        for (i, primitive) in primitives.into_iter().enumerate() {
            match primitive {
                ClippedPrimitive { clip_rect, primitive } => {
                    match primitive {
                        Primitive::Mesh(mut mesh) => {
                            if x.len() <= i {
                                let mesh = sk.mesh_create();
                                x.push(mesh);
                            }
                            let m = x.get_mut(i).unwrap();
                            let mut verts = vec![];
                            for vertex in mesh.vertices {
                                verts.push(Vert {
                                    pos: Vec3::new(vertex.pos.x, vertex.pos.y, 0.0),
                                    norm: Vec3::new(0.0, 0.0, -1.0),
                                    uv: Vec2::new(vertex.uv.x, vertex.uv.y),
                                    col: Color32::new(vertex.color.r(), vertex.color.g(), vertex.color.b(), vertex.color.a()),
                                })
                            }
                            sk.mesh_set_data(&m, &verts, &mesh.indices, true);
                            if let Some((texture, material)) = self.textures.get(&mesh.texture_id) {
                                sk.material_set_texture(material, "diffuse", texture);
                                sk.material_set_cull(material, CullMode::None);
                                sk.material_set_depth_test(material, DepthTest::LessOrEq);
                                sk.material_set_depth_write(material, false);
                                sk.material_set_queue_offset(material, 200);
                                let matrix = Mat4::from_scale_rotation_translation(Vec3::new(-0.005, -0.005, -0.01), Quat::IDENTITY, pos);
                                sk.mesh_draw(m, material, matrix, Color128::new_rgb(1.0, 1.0, 1.0), RenderLayer::LAYER_ALL);
                            }
                        }
                        Primitive::Callback(a) => {
                            panic!()
                        }
                    }
                }
            }
        }
    }
}/*let mut prev_ind = vec![];
                            let len = mesh.indices.len();
                            z = 0.0;
                            for i in (0..len).step_by(3) {
                                if !prev_ind.contains(&mesh.indices[i]) && !prev_ind.contains(&mesh.indices[i+1]) && !prev_ind.contains(&mesh.indices[i+2]) {
                                    z += 0.1;
                                }
                                if let Some(v) = verts.get_mut(i) {
                                    v.pos.z = z;
                                }
                                if let Some(v) = verts.get_mut(i+1) {
                                    v.pos.z = z;
                                }
                                if let Some(v) = verts.get_mut(i+2) {
                                    v.pos.z = z;
                                }
                            }
*/

/*let mut inds = vec![];
                            let mut index = 0;
                            for mut i in 0..mesh.indices.len() {
                                if (index >= 20) {
                                    if (index + 1 > mesh.indices.len()) {
                                        break;
                                    }
                                    inds.push(mesh.indices[index]);
                                    index += 1;
                                    continue;
                                }
                                if (index + 2 > mesh.indices.len()) {
                                    break;
                                }
                                inds.push(mesh.indices[index+2]);
                                inds.push(mesh.indices[index+1]);
                                inds.push(mesh.indices[index]);
                                index += 3;
                            }*/