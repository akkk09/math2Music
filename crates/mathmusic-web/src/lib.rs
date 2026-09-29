use mathmusic_core::parse;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext,CanvasRenderingContext2d,GainNode,HtmlCanvasElement,HtmlElement,HtmlInputElement,OscillatorNode};

thread_local!{static AUDIO:RefCell<Option<AudioState>>=const{RefCell::new(None)};}
struct AudioState{_context:AudioContext,_gain:GainNode,osc:Option<OscillatorNode>}

fn document()->Result<web_sys::Document,JsValue>{web_sys::window().ok_or_else(||JsValue::from_str("window unavailable"))?.document().ok_or_else(||JsValue::from_str("document unavailable"))}
fn el<T:JsCast>(id:&str)->Result<T,JsValue>{document()?.get_element_by_id(id).ok_or_else(||JsValue::from_str(&format!("missing #{id}")))?.dyn_into::<T>().map_err(|_|JsValue::from_str("invalid element type"))}
fn status(s:&str){if let Ok(n)=el::<HtmlElement>("status"){n.set_inner_text(s)}}

fn draw(expr:&str)->Result<(),JsValue>{
 let canvas:HtmlCanvasElement=el("graph")?;let ctx:CanvasRenderingContext2d=canvas.get_context("2d")?.ok_or_else(||JsValue::from_str("2d context unavailable"))?.dyn_into()?;
 let w=canvas.width()as f64;let h=canvas.height()as f64;ctx.set_fill_style_str("#0b0d12");ctx.fill_rect(0.,0.,w,h);
 ctx.set_stroke_style_str("#252a35");ctx.set_line_width(1.);ctx.begin_path();ctx.move_to(0.,h/2.);ctx.line_to(w,h/2.);ctx.stroke();
 let Ok(function)=parse(expr)else{return Ok(())};let samples=function.sample(-std::f64::consts::PI*2.,std::f64::consts::PI*2.,w.max(2.)as usize);
 let max=samples.iter().map(|v|v.abs()).fold(0.25,f64::max).min(100.);ctx.set_stroke_style_str("#65d9ff");ctx.set_line_width(2.);ctx.begin_path();
 for(i,y)in samples.iter().enumerate(){let px=i as f64/(samples.len()-1)as f64*w;let py=h/2.-(*y/max)*(h*0.42);if i==0{ctx.move_to(px,py)}else{ctx.line_to(px,py)}}ctx.stroke();Ok(())
}
fn stop_audio(){AUDIO.with(|a|if let Some(s)=a.borrow_mut().as_mut(){if let Some(o)=s.osc.take(){let _=o.stop();}})}
fn play(expr:&str,freq:f64,gain:f64)->Result<(),JsValue>{stop_audio();let context=AudioContext::new()?;let g=GainNode::new(&context)?;g.gain().set_value(gain as f32);g.connect_with_audio_node(&context.destination())?;let osc=context.create_oscillator()?;osc.frequency().set_value(freq as f32);osc.connect_with_audio_node(&g)?;osc.start()?;AUDIO.with(|a|*a.borrow_mut()=Some(AudioState{_context:context,_gain:g,osc:Some(osc)}));status(&format!("playing {expr} @ {freq:.0} Hz"));Ok(())}

#[wasm_bindgen(start)]
pub fn start()->Result<(),JsValue>{
 console_error_panic_hook::set_once();let expression:HtmlInputElement=el("expression")?;draw(&expression.value())?;
 let input_expr=expression.clone();let input=Closure::<dyn FnMut(web_sys::Event)>::new(move |_|{let _=draw(&input_expr.value());});expression.add_event_listener_with_callback("input",input.as_ref().unchecked_ref())?;input.forget();
 let play_expr=expression.clone();let button:HtmlElement=el("play")?;let cb=Closure::<dyn FnMut(web_sys::Event)>::new(move |_|{let f:HtmlInputElement=el("frequency").unwrap();let g:HtmlInputElement=el("gain").unwrap();let freq=f.value().parse().unwrap_or(220.);let gain=g.value().parse().unwrap_or(0.15);if let Err(e)=play(&play_expr.value(),freq,gain){status(&format!("audio error: {e:?}"));}});button.add_event_listener_with_callback("click",cb.as_ref().unchecked_ref())?;cb.forget();
 let stop:HtmlElement=el("stop")?;let cb=Closure::<dyn FnMut(web_sys::Event)>::new(move |_|{stop_audio();status("stopped")});stop.add_event_listener_with_callback("click",cb.as_ref().unchecked_ref())?;cb.forget();
 let demo_expr=expression.clone();let demo:HtmlElement=el("demo")?;let cb=Closure::<dyn FnMut(web_sys::Event)>::new(move |_|{demo_expr.set_value("sin(x) + 0.5sin(3x)");let _=draw(&demo_expr.value());status("loaded harmonic demo")});demo.add_event_listener_with_callback("click",cb.as_ref().unchecked_ref())?;cb.forget();
 status("ready");Ok(())
}
