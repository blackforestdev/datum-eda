struct Scene { resolution: vec4<f32>, viewport_origin: vec4<f32>, viewport_size: vec4<f32>, camera_center_scale: vec4<f32> };
@group(0) @binding(0) var<uniform> scene: Scene;
@group(1) @binding(0) var<storage,read> ranges: array<vec4<u32>>;
struct QuadIn { @builtin(vertex_index) vertex:u32, @location(0) point:vec2<f32>, @location(1) color:vec3<f32> };
struct StrokeIn { @builtin(vertex_index) vertex:u32, @builtin(instance_index) instance:u32, @location(0) a:vec2<f32>, @location(1) b:vec2<f32>, @location(2) material_floor:vec4<f32>, @location(3) nominal:f32 };
struct Out {
 @builtin(position) position:vec4<f32>, @location(0) material:vec3<f32>,
 @location(1) local:vec2<f32>, @location(2) @interpolate(flat) capsule:vec2<f32>,
 @location(3) @interpolate(flat) label:u32, @location(4) logical:vec2<f32>,
};
fn group(index:u32, stroke:bool)->u32 {
 let first=select(2u,ranges[0].z,stroke); let count=select(ranges[0].x,ranges[0].y,stroke);
 var lo=first; var hi=first+count;
 loop { if (lo>=hi) {break;} let mid=lo+(hi-lo)/2u;
  if (ranges[mid].y<=index) {lo=mid+1u;} else {hi=mid;}
 }
 if (lo<first+count && ranges[lo].x<=index) {return ranges[lo].z;} return 0u;
}
fn position(p:vec2<f32>)->vec4<f32> {
 return vec4<f32>(p.x/scene.resolution.x*2.0-1.0,1.0-p.y/scene.resolution.y*2.0,0.0,1.0);
}
fn project(p:vec2<f32>)->vec2<f32> {
 return scene.viewport_origin.xy+scene.viewport_size.xy*0.5+(p-scene.camera_center_scale.xy)*scene.camera_center_scale.z;
}
@vertex fn vs_quad(i:QuadIn)->Out {
 var o:Out; o.logical=project(i.point); o.position=position(o.logical); o.material=i.color;
 o.local=vec2<f32>(0.0); o.capsule=vec2<f32>(0.0,-1.0); o.label=group(i.vertex,false); return o;
}
@vertex fn vs_stroke(i:StrokeIn)->Out {
 let endpoint_b=i.vertex==1u || i.vertex==2u || i.vertex==4u;
 let positive=i.vertex==0u || i.vertex==1u || i.vertex==3u;
 let center=select(i.a,i.b,endpoint_b); let d=i.b-i.a;
 let tangent=d/max(length(d),1.0); let normal=vec2<f32>(-tangent.y,tangent.x);
 let radius=max(i.nominal*scene.camera_center_scale.z,i.material_floor.w)*0.5;
 let half_length=length(d)*scene.camera_center_scale.z*0.5;
 let across=select(-1.0,1.0,positive)*radius;
 let p=project(center)+tangent*select(-1.0,1.0,endpoint_b)*radius+normal*across;
 var o:Out; o.logical=p; o.position=position(p); o.material=i.material_floor.xyz;
 o.local=vec2<f32>(select(-1.0,1.0,endpoint_b)*(half_length+radius),across);
 o.capsule=vec2<f32>(half_length,radius); o.label=group(i.instance,true); return o;
}
fn covered(i:Out)->bool {
 if (i.capsule.y<0.0) {return true;}
 let cap=max(abs(i.local.x)-i.capsule.x,0.0);
 return cap*cap+i.local.y*i.local.y<=i.capsule.y*i.capsule.y;
}
fn encode(label:u32)->vec4<f32> {
 return vec4<f32>(f32(label&255u),f32((label>>8u)&255u),f32((label>>16u)&255u),f32((label>>24u)&255u))/255.0;
}
fn decode(value:vec4<f32>)->u32 {
 let b=vec4<u32>(round(value*255.0)); return b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u);
}
fn linear(c:vec3<f32>)->vec3<f32> {
 return select(pow((c+vec3<f32>(0.055))/1.055,vec3<f32>(2.4)),c/12.92,c<=vec3<f32>(0.04045));
}
struct MaskOut { @location(0) label:vec4<f32>, @location(1) material:vec4<f32> };
@fragment fn fs_mask(i:Out,@builtin(sample_index) sample:u32)->MaskOut {
 if (!covered(i)) {discard;}
 var o:MaskOut; o.label=encode(i.label);
 o.material=vec4<f32>(linear(min(i.material*1.08,vec3<f32>(1.0))),1.0);
 return o;
}
