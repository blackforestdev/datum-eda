struct Screen { resolution:vec2<f32>, pad:vec2<f32> };
@group(0) @binding(0) var<uniform> screen:Screen;
@group(2) @binding(2) var<storage,read> band:array<vec4<u32>>;
@group(2) @binding(3) var material:texture_multisampled_2d<f32>;
@group(2) @binding(0) var mask:texture_multisampled_2d<f32>;
@group(2) @binding(1) var<storage,read> ranges:array<vec4<u32>>;
struct Out { @builtin(position) position:vec4<f32>, @location(0) logical:vec2<f32> };
@vertex fn vs_cue(@builtin(vertex_index) vertex:u32)->Out {
 let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0)); var o:Out; o.position=vec4<f32>(p[vertex],0.0,1.0);
 o.logical=vec2<f32>((p[vertex].x+1.0)*0.5,(1.0-p[vertex].y)*0.5)*screen.resolution; return o;
}
fn decode(value:vec4<f32>)->u32 {
 let b=vec4<u32>(round(value*255.0)); return b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u);
}
fn label(p:vec2<i32>,sample:u32)->u32 {
 if (any(p<vec2<i32>(0)) || any(p>=vec2<i32>(textureDimensions(mask)))) {return 0u;}
 return decode(textureLoad(mask,p,i32(sample)));
}
fn selected(label:u32)->bool {return label!=0u && label!=0xffffffffu;}
@fragment fn fs_cue(i:Out,@builtin(sample_index) sample:u32)->@location(0) vec4<f32> {
 if (!datum_damaged(i.position)) {discard;}
 let p=vec2<i32>(i.logical-vec2<f32>(0.0,f32(band[0].x)-2.0)); let center=label(p,sample);
 // Authored voids stay open, including holes surrounded by selected material.
 if (center==0xffffffffu) {discard;}
 var outside=false; var edge_distance=3.0;
 for (var y=-2; y<=2; y+=1) { for (var x=-2; x<=2; x+=1) {
   let distance=length(vec2<f32>(f32(x),f32(y)));
   if (distance>2.0 || distance==0.0) {continue;}
   let neighbor=label(p+vec2<i32>(x,y),sample);
   if (selected(neighbor) && neighbor!=center) {outside=true;}
   if (selected(center) && neighbor!=center) {edge_distance=min(edge_distance,distance);}
 }}
 let accent=vec3<f32>(206.0,90.0,146.0)/255.0;
 let c=select(pow((accent+vec3<f32>(0.055))/1.055,vec3<f32>(2.4)),accent/12.92,accent<=vec3<f32>(0.04045));
 if (outside) {return vec4<f32>(c,1.0);}
 if (!selected(center) || (ranges[0].w&1u)!=0u || edge_distance>2.0) {discard;}
 return vec4<f32>(c,0.30*(1.0-smoothstep(0.0,2.0,edge_distance-0.5)));
}

@fragment fn fs_lift(i:Out,@builtin(sample_index) sample:u32)->@location(0) vec4<f32> {
 if (!datum_damaged(i.position)) {discard;}
 let p=vec2<i32>(i.logical-vec2<f32>(0.0,f32(band[0].x)-2.0));
 let color=textureLoad(material,p,i32(sample));
 if (color.a==0.0) {discard;}
 return color;
}
