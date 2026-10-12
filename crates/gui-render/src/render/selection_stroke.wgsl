struct Screen { resolution: vec2<f32>, pad: vec2<f32> };
@group(0) @binding(0) var<uniform> screen: Screen;
@group(2) @binding(0) var<storage, read> clips: array<vec4<f32>>;
struct In {
 @builtin(vertex_index) vertex: u32,
 @location(0) a: vec2<f32>, @location(1) b: vec2<f32>,
 @location(2) material_width: vec4<f32>, @location(3) clip: u32, @location(4) flags: u32,
};
struct Out {
 @builtin(position) position: vec4<f32>,
 @location(0) local: vec2<f32>,
 @location(1) @interpolate(flat) length_radius: vec2<f32>,
 @location(2) @interpolate(flat) material: vec3<f32>,
 @location(3) @interpolate(flat) clip: vec4<f32>,
 @location(4) @interpolate(flat) contrast: vec2<f32>,
};
@vertex fn vs_main(v:In)->Out {
 let direction=v.b-v.a; let length=length(direction);
 let u=select(vec2<f32>(1.0,0.0),direction/max(length,0.000001),length>0.000001); let n=vec2<f32>(-u.y,u.x);
 let radius=v.material_width.w*0.5; let extent=radius+2.0;
 let corners=array<vec2<f32>,6>(vec2<f32>(-extent,-extent),vec2<f32>(length+extent,-extent),vec2<f32>(length+extent,extent),vec2<f32>(-extent,-extent),vec2<f32>(length+extent,extent),vec2<f32>(-extent,extent));
 let local=corners[v.vertex]; let p=v.a+u*local.x+n*local.y;
 var o:Out; o.position=vec4<f32>(p.x/screen.resolution.x*2.0-1.0,1.0-p.y/screen.resolution.y*2.0,0.0,1.0);
 o.local=local;o.length_radius=vec2<f32>(length,radius);o.material=v.material_width.xyz;o.clip=clips[v.clip];o.contrast=vec2<f32>(f32(v.flags&1u),f32(v.flags>>1u));return o;
}
fn linear(c:vec3<f32>)->vec3<f32> {
 return select(pow((c+vec3<f32>(0.055))/1.055,vec3<f32>(2.4)),c/12.92,c<=vec3<f32>(0.04045));
}
fn eligible(v:Out)->bool {
 return datum_damaged(v.position) && v.position.x>=v.clip.x && v.position.y>=v.clip.y && v.position.x<v.clip.z && v.position.y<v.clip.w;
}
fn distance(v:Out)->f32 {return length(vec2<f32>(v.local.x-clamp(v.local.x,0.0,v.length_radius.x),v.local.y));}
@fragment fn fs_lift(v:Out,@builtin(sample_index) sample_index:u32)->@location(0) vec4<f32> {
 if (!eligible(v) || v.contrast.y>0.5 || distance(v)>v.length_radius.y) {discard;}
 return vec4<f32>(linear(min(v.material*1.08,vec3<f32>(1.0))),1.0);
}
@fragment fn fs_cue(v:Out,@builtin(sample_index) sample_index:u32)->@location(0) vec4<f32> {
 if (!eligible(v)) { discard; }
 let d=distance(v); let radius=v.length_radius.y;
 if (d>radius+2.0) {discard;}
 let accent=linear(vec3<f32>(206.0,90.0,146.0)/255.0);
 // Minimum cues have no material lift or glow. The ring remains open;
 // NoConnect retains its X rather than becoming a junction.
 if (v.contrast.y>1.5) {
   if (d>radius) {discard;} return vec4<f32>(accent,1.0);
 }
 if (v.contrast.y>0.5) {
   if (d<radius-1.0 || d>radius+1.0) {discard;}
   return vec4<f32>(accent,1.0);
 }
 if (d>=radius) {return vec4<f32>(accent,1.0);}
 // Glow is supplementary and translucent so an overlapping proposal survives.
 if (v.contrast.x>0.5 || radius-d>=2.0) {discard;}
 let opacity=0.30*(1.0-smoothstep(0.0,2.0,radius-d));
 return vec4<f32>(accent,opacity);
}
