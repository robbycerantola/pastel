use orbimage;
use orbclient;
use orbtk::{Color, Rect, Renderer, Window}; //Action, Button, Image, Label, Menu, Point, ProgressBar,ControlKnob,Toolbar, ToolbarIcon,Separator,TextBox, Window,InnerWindow,, ColorSwatch
use orbclient::EventOption;

use std::f32::consts::PI;

use CANVASOFFSET;

//  which ones to be added directly to orbclient ?
pub trait AddOnsToOrbimage {
    fn fill(&mut self, x: i32 , y: i32, color: Color);
    fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
    fn flood_fill_scanline(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
    fn flood_fill_line(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
    fn pixcol(&self, x:i32, y:i32) -> Color;
    fn pixraw(&self, x:i32, y:i32) -> u32;
    //fn interact_rect(&mut self, x: i32 , y: i32, color: Color, filled: bool, width: i32, window: &mut Window) ->Option<Rect>;
    fn interact_line(&mut self, x: i32 , y: i32, color: Color,width: i32, antialias: bool, window: &mut Window);
    fn interact_circle(&mut self, x: i32 , y: i32, color: Color, window: &mut Window) -> Option<(i32,f32)>;
    fn interact_paste(&mut self, x: i32 , y: i32, opacity: u8, buffer: orbimage::Image, window: &mut Window);
    fn select_rect(&mut self, x: i32 , y: i32, window: &mut Window) ->Option<Rect>;
    fn new_select_rect(&mut self, x: i32 , y: i32, color: Color, pattern: i32, window: &mut Window) ->Option<Rect>;
    fn copy_selection(&self, x: i32,y: i32,w: u32, h: u32) -> orbimage::Image;
    fn paste_selection(&mut self, x: i32, y:i32, opacity: u8, buffer: orbimage::Image);
    fn smooth_circle(&mut self, x: i32, y:i32, size: u32, color: Color);
    fn wu_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color);
    fn wu_circle(&mut self, x0: i32, y0: i32, radius: i32, color: Color);
    fn polygon(&mut self, x: i32, y: i32, r: i32, sides: u32, angle: f32,color: Color, antialias: bool);
    
}

impl AddOnsToOrbimage for orbimage::Image {
/*
    ///return rgba color of image pixel at position (x,y)  NOT SAFE if x y are bigger than current image size, but very quick.
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.data()[p as usize];
        rgba
    }
*/
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = (self.width()as i32 * y + x) as usize;
        if p < self.data().len() {
            let rgba = self.data()[p];
            return rgba
        }else { return Color::rgba(0,0,0,0)}
    }
    
    fn pixraw (&self, x:i32, y:i32) -> u32 {
        self.pixcol(x,y).data 
    }
    ///wrapper for flood fill 
    fn fill(&mut self, x: i32, y: i32 , color: Color) {
        //get current pixel color 
        let rgba = self.pixcol(x,y);
        //self.flood_fill4(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
        //println!("Old color {}", rgba.data);
        self.flood_fill_scanline(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
    }

    /*Recursive 4-way floodfill works with transparency but be aware of stack overflow !! 
    credits to http://lodev.org/cgtutor/floodfill.html
    Added stacker crate to mitigate overflow issue , does it work also for Redox?
    */
    fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        //stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        
        if x >= 0 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
            
            self.pixel(x,y, Color{data:new_color});
            
            self.flood_fill4(x+1,y,new_color,old_color);
            self.flood_fill4(x-1,y,new_color,old_color);
            self.flood_fill4(x,y+1,new_color,old_color);
            self.flood_fill4(x,y-1,new_color,old_color);
        }
        //});
    }
    
    ///stack friendly and fast floodfill algorithm that works with transparency too 
    fn flood_fill_scanline( &mut self, x:i32, y:i32, new_color: u32, old_color:u32) {
        if old_color == new_color {
            return;
        }
        if self.pixcol(x,y).data  != old_color  {
            return;
        }
        
        let w = self.width() as i32;
        let h = self.height() as i32;
        
        //draw current scanline from start position to the right
        let mut x1 = x;
        
        while x1 < w && self.pixcol(x1,y).data  == old_color  {
            self.pixel(x1,y,Color{data:new_color});
            x1 +=1;
        } 
        //get resulted color because of transparency and use this for comparison 
        let res_color = self.pixcol(x,y).data;
        
        //draw current scanline from start position to the left
        x1 = x -1;
        
        while x1 >= 0 && self.pixcol(x1,y).data  == old_color  {
            self.pixel(x1,y,Color{data:new_color});
            x1 += -1;
          }
        
        //test for new scanlines above
        x1 = x;
        
        while x1 < w && self.pixcol(x1,y).data  == res_color  { 
            if y > 0 && self.pixcol(x1,y-1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += 1;
          }
        x1 = x - 1;
        while x1 >= 0 && self.pixcol(x1,y).data == res_color {
            if y > 0 && self.pixcol(x1,y - 1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += -1;
          }
         
         //test for new scanlines below
        x1 = x;
        while x1 < w && self.pixcol(x1,y).data == res_color  {
            //println!("Test below {} {} ", self.pixcol(x1,y).data,old_color);
            if y < (h - 1) && self.pixcol(x1,y + 1).data == old_color {
                self.flood_fill_scanline(x1, y + 1, new_color, old_color);
            }
            x1 +=1;
        }
        x1 = x - 1;
        while x1 >= 0 && self.pixcol(x1,y).data == res_color {
            if y < (h - 1) && self.pixcol(x1,y + 1).data == old_color {
                self.flood_fill_scanline(x1, y + 1, new_color, old_color);
            }
            x1 += -1;
        }
    }

    fn flood_fill_line(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        if x >= 1 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
           
           let mut x1=x; 
           loop {
                if x1>=0 && x1< self.width() as i32 && self.pixcol(x1,y).data == old_color{
                    self.pixel(x1,y, Color{data:new_color});
                    x1 +=1;
                } else {break}  
            }
           x1=x-1; 
           loop {
                if x1>=0 && x1< self.width() as i32 && self.pixcol(x1,y).data == old_color{
                    self.pixel(x1,y, Color{data:new_color});
                    x1 +=-1;
                } else {break}  
            }
            self.flood_fill_line(x,y+1,new_color,old_color);
            self.flood_fill_line(x,y-1,new_color,old_color);
        }
    }

    /// draws antialiased line
     //adapted from https://rosettacode.org/wiki/Xiaolin_Wu's_line_algorithm#C.23   
    fn wu_line (&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        
        let mut x0 = x0 as f64;
        let mut y0 = y0 as f64;
        let mut x1 = x1 as f64;
        let mut y1 = y1 as f64;
        let r = color.r();
        let g = color.g();
        let b = color.b();
        let a = color.a() as f64;
        
        fn ipart (x: f64) -> i32 {
            x.trunc() as i32
        }
        fn round (x: f64) -> i32 {
            ipart(x+0.5) as i32
        }
        fn fpart (x: f64) -> f64 {
            if x <0.0 { return 1.0-(x-x.floor());}
            x-x.floor() 
        }
        fn rfpart(x: f64) -> f64 {
            1.0-fpart(x)
        }
        fn chkalpha (mut alpha :f64) -> u8 {
             if alpha > 255.0 { alpha = 255.0};
             if alpha < 0.0 {alpha = 0.0};
             alpha as u8
        }
        
        let steep :bool = (y1-y0).abs() > (x1-x0).abs();
        let mut temp;
        if steep {
            temp = x0; x0 = y0; y0 = temp;
            temp = x1; x1 = y1; y1 = temp;
        }
        if x0 > x1 {
            temp = x0; x0 = x1; x1 = temp;
            temp = y0; y0 = y1; y1 = temp;
        }
        let dx = x1 -x0;
        let dy = y1- y0;
        let gradient = dy/dx;
        
        let mut xend: f64 = (x0 as f64).round() ;
        let mut yend: f64 = y0 + gradient * (xend - x0);
        let mut xgap: f64 = rfpart(x0+0.5);
        let xpixel1 = xend as i32;
        let ypixel1 = (ipart (yend)) as i32;
        
        if steep {
            self.pixel(ypixel1, xpixel1, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(ypixel1+1, xpixel1, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }else{
            self.pixel(xpixel1, ypixel1, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(xpixel1+1, ypixel1, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }
        let mut intery :f64 = yend + gradient;
        xend = x1.round();
        yend = y1 + gradient * (xend-x1);
        xgap = fpart(x1 + 0.5);
        let xpixel2 = xend as i32;
        let ypixel2 = ipart(yend) as i32;
        if steep {
            self.pixel(ypixel2, xpixel2, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(ypixel2+1, xpixel2, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }else{
            self.pixel(xpixel2, ypixel2, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(xpixel2+1, ypixel2, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }
        if steep {
            for x in (xpixel1+1)..(xpixel2) {
                self.pixel(ipart(intery) as i32 , x, Color::rgba(r,g,b,chkalpha(a*rfpart(intery))));
                self.pixel(ipart(intery) as i32 + 1, x, Color::rgba(r,g,b,chkalpha(a*fpart(intery))));
                intery += gradient;
            }
        }else{
            for x in (xpixel1+1)..(xpixel2) {
                self.pixel(x, ipart(intery) as i32, Color::rgba(r,g,b,chkalpha(a*rfpart(intery))));
                self.pixel(x, ipart(intery) as i32 + 1, Color::rgba(r,g,b,chkalpha(a*fpart(intery))));
                intery += gradient;
            } 
        }           
    }

    ///Draws antialiased circle
    fn wu_circle (&mut self, x0: i32, y0: i32, radius: i32, color: Color){
        let r = color.r();
        let g = color.g();
        let b = color.b();
        let a = color.a();
        let mut y =0;
        let mut x = radius;
        let mut d =0_f64;
        
        self.pixel (x0+x,y0+y,color);
        self.pixel (x0-x,y0-y,color);
        self.pixel (x0+y,y0-x,color);
        self.pixel (x0-y,y0+x,color);
        
        while x > y {
            let di = dist(radius,y);
            if di < d { x -= 1;}
            let col = Color::rgba(r,g,b,(a as f64*(1.0-di)) as u8);
            let col2 = Color::rgba(r,g,b,(a as f64*di) as u8);
            
            self.pixel(x0+x, y0+y, col);
            self.pixel(x0+x-1, y0+y, col2);//-
            self.pixel(x0-x, y0+y, col);
            self.pixel(x0-x+1, y0+y, col2);//+
            self.pixel(x0+x, y0-y, col);
            self.pixel(x0+x-1, y0-y, col2);//-
            self.pixel(x0-x, y0-y, col);
            self.pixel(x0-x+1, y0-y, col2);//+
            
            self.pixel(x0+y, y0+x, col);
            self.pixel(x0+y, y0+x-1, col2);
            self.pixel(x0-y, y0+x, col);
            self.pixel(x0-y, y0+x-1, col2);
            self.pixel(x0+y, y0-x, col);
            self.pixel(x0+y, y0-x+1, col2);
            self.pixel(x0-y, y0-x, col);
            self.pixel(x0-y, y0-x+1, col2);
            d = di;
            y += 1;
        }
        
        fn dist(r: i32, y: i32) -> f64{
            let x :f64 = ((r*r-y*y)as f64).sqrt();
            x.ceil()-x
        }
    }

    ///Draws a regular polygon
    fn polygon(&mut self, x0: i32, y0: i32, r: i32, sides: u32, angle: f32, color: Color, antialias: bool ) {
        let mut x:Vec<i32> = Vec::new();
        let mut y:Vec<i32> = Vec::new();
        let i :usize = 0;
        let sides = sides as usize;
        //find vertices
        for i in 0..sides+1 {
            let t :f32 =angle + 2.0*PI* i as f32 /sides as f32;
            x.push((r as f32 * t.cos()) as i32 + x0);
            y.push((r as f32 * t.sin()) as i32 + y0);
        }
        
        if antialias {
        for i in 0..sides {
            self.wu_line(x[i],y[i],x[i+1],y[i+1],color);
        }
        self.wu_line(x[sides],y[sides],x[0],y[0],color);    
        }else{
        for i in 0..sides-1 {
            self.line(x[i],y[i],x[i+1],y[i+1],color);
        }
        self.line(x[sides],y[sides],x[0],y[0],color);
        }
            
            
    }

    ///crop new image from current image (copy) tranforming pure white into transparent
    fn copy_selection(&self, x: i32,y: i32,w: u32, h: u32) -> orbimage::Image {

        let mut vec = vec![];
        let mut col : Color;
        
        for y1 in y..y+h as i32 {
            for x1 in x..x+w as i32 {
                col=self.pixcol(x1,y1);
                if col.r()==255 && col.g()==255 && col.b()==255 {
                    col = Color::rgba(0,0,0,0);
                }
                vec.push(col);
            }
        }
        //println!("buffer {:?}",&vec);
        orbimage::Image::from_data(w ,h ,vec.into_boxed_slice()).unwrap()
    }

    ///draws an image into current image starting at x,y (paste) with transparency
    fn paste_selection (&mut self, x: i32, y:i32, opacity: u8, buffer: orbimage::Image, ){
        
        let w = buffer.width() as i32;
        let h = buffer.height() as i32;
        let xc=x-w/2; //center buffer at cursor 
        let yc=y-h/2;
        let data = buffer.into_data();
        let mut i:usize = 0;
        let mut r;
        let mut g;
        let mut b;
        let mut a;
        let x1:i32;
        let y1:i32;
        
        for y1 in yc..yc+h {
            for x1 in xc..xc+w {
                if i < data.len(){
                    r = data[i].r();
                    g = data[i].g();
                    b = data[i].b();
                    a = data[i].a();
                    if a != 0 {a = opacity}
                    self.pixel(x1,y1,Color::rgba(r,g,b,a));
                }
                i += 1;
            }
        }
    }
    
    //experimental smooth brush : work in progress....
    fn smooth_circle (&mut self, x: i32, y:i32, size: u32, color: Color) {
        //let mut sb= orbimage::Image::from_color(2*size, 2*size, Color::rgba(255,255,255,0));
        let sb = orbimage::Image::from_path("smooth_circle_yellow.png").unwrap();
        
        let r = color.r();
        let g = color.g();
        let b = color.b();
        let a = color.a();
        
        /*
        for n in 0..size {
            //sb.circle(size as i32 , size as i32 , ((size -n) as i32), Color::rgba(r,g,b,(2*n) as u8));
            sb.pixel(n as i32,n as i32, Color::rgba(r,g,b,(4*n)as u8)); //Does NOT work as intended!!
            //sb.pixel(n as i32,n as i32, Color::rgba(r,g,b,(4*n)as u8));
        }
        */
        //self.paste_selection(x,y,80,sb);
        self.image(x,y,sb.width(),sb.height(),sb.data());
        //println!("{:?}",sb.data());
        
    }

    /// interactive selection (rectangle)  
    fn select_rect(&mut self, x: i32 , y: i32, window: &mut Window) ->Option<Rect> {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx = 0;
         let mut ly = 0;
         let mut x = x;
         let mut y = y;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                   EventOption::Key(key_event) => {println!("{:?}",key_event); break 'events},
                EventOption::Quit(_quit_event) => break 'events,
             EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                       EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events;
                                                };
                                                if w {
                                                    orbclient.rect_marquee(x,
                                                    y+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    Color::rgba(100, 100, 100, 0),3);
                                                }
                                                w=true;
                                                
                                                orbclient.rect_marquee(x,
                                                y+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                Color::rgba(100, 100, 100, 0),3);
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                
                                                orbclient.sync();
                                                },
                    EventOption::Button(btn) => {if btn.left {
                                                    
                                                    if lx < x {
                                                        let tmp =x; x= lx; lx = tmp;
                                                    } 
                                                    
                                                    if ly < y {
                                                        let tmp = y; y=ly; ly= tmp;
                                                    }
                                                    
                                                    let dx=lx-x;
                                                    let dy=ly-y;
                                                    //println!("{} {} {} {}",x,y,dx,dy);
                                                    return Some(Rect::new(x,y,dx as u32, dy as u32))
                                                }
                                                if btn.right{
                                                              break 'events;
                                                            //TODO show menu with actions upon selection
                                                }
                                                },
                                event_option => if cfg!(feature = "debug"){
                                                    println!("{:?}", event_option)
                                                }else{ ()}
                }
          }
        }
        None  
    }

    /// interactive selection (rectangle) pattern is an integer where 1 means continuuos line , 2 dotted line , 3 dotted line more spaced and so on 
    fn new_select_rect(&mut self, x: i32 , y: i32, color: Color, pattern: i32, window: &mut Window) ->Option<Rect> {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx = 0;
         let mut ly = 0;
         let mut x = x;
         let mut y = y;
         let mut w = false;
         
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                   EventOption::Key(key_event) => {println!("{:?}",key_event); break 'events},
                EventOption::Quit(_quit_event) => break 'events,
             EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                       EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events;
                                                };
                                                if w {
                                                    orbclient.rect_marquee(x,
                                                    y+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    color,
                                                    pattern);
                                                }
                                                w=true;
                                                
                                                orbclient.rect_marquee(x,
                                                y+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                color,
                                                pattern);
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                
                                                orbclient.sync();
                                                },
                    EventOption::Button(btn) => {if btn.left {
                                                    
                                                    if lx < x {
                                                        let tmp =x; x= lx; lx = tmp;
                                                    } 
                                                    
                                                    if ly < y {
                                                        let tmp = y; y=ly; ly= tmp;
                                                    }
                                                    
                                                    let dx=lx-x;
                                                    let dy=ly-y;
                                                    //println!("{} {} {} {}",x,y,dx,dy);
                                                    return Some(Rect::new(x,y,dx as u32, dy as u32))
                                                }
                                                if btn.right{
                                                              break 'events;
                                                            //TODO show menu with actions upon selection
                                                }
                                                },
                                event_option => if cfg!(feature = "debug"){
                                                    println!("{:?}", event_option)
                                                }else{ ()}
                }
          }
        }
        None  
    }
/*
    /// draws interactive rectangle 
    fn interact_rect(&mut self, x: i32 , y: i32, color: Color,filled:bool, width: i32, window: &mut Window) ->Option<Rect> {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx = 0;
         let mut ly = 0;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => {println!("{:?}",key_event); break 'events;},
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };
                                                if w {
                                                    orbclient.rect_marquee(x,
                                                    y+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    Color::rgba(100, 100, 100, 0),0);
                                                }
                                                w=true;
                                                
                                                orbclient.rect_marquee(x,
                                                y+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                Color::rgba(100, 100, 100, 0),0);
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                
                                                orbclient.sync();
                                                   
                                                },
                    EventOption::Button(btn) => {if btn.left {
                                                    if filled {
                                                        let dx=lx-x;
                                                        let dy=ly-y;
                                                        if dx >0 && dy>0 {
                                                            self.rect(x ,y,dx as u32, dy as u32 ,color);
                                                        }
                                                        if dx<0 && dy > 0 {
                                                            self.rect(x+dx ,y ,-dx as u32, dy as u32, color);
                                                        }
                                                        if dx<0 && dy <0 {
                                                            self.rect(x+dx ,y+dy ,-dx as u32, -dy as u32, color);
                                                        }
                                                        if dx>0 && dy <0 {
                                                            self.rect(x ,y+dy ,dx as u32, (-dy) as u32, color);
                                                        }
                                                        break 'events
                                                    } else {
                                                        for i in 0..width {
                                                            self.line(x-i,y-i,lx+i,y-i,color);
                                                            self.line(lx+i,y-i,lx+i,ly+i,color);
                                                            self.line(lx+i,ly+i,x-i,ly+i,color);
                                                            self.line(x-i,ly+i,x-i,y-i,color);
                                                        
                                                        }
                                                        break 'events
                                                    }
                                                }
                                                if btn.right{
                                                        break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("{:?}", event_option)}
                                    else{ ()}
                }
          }
          
        }
        None
    }
*/ 
    /// by drawing an interactive circle in preview window , return a tuple with radius and cursor angular position  
    fn interact_circle(&mut self, x: i32 , y: i32, color: Color, window: &mut Window) -> Option<(i32,f32)>{
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut w = false;
         let mut dx = 0_i32;
         let mut dy = 0_i32;
         let mut r = 0_i32;
         let mut r_old = 0_i32;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => {println!("Event:{:?}",key_event); break 'events},
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };
                                                if w {
                                                    orbclient.circle_marquee(x, y+CANVASOFFSET, r_old, Color::rgba(100, 100, 100, 0)); 
                                                }
                                                w=true;
                                                r = dx.pow(2)+dy.pow(2);
                                                r = ((r as f64).sqrt()) as i32;
                                                r_old = r;
                                                
                                                orbclient.circle_marquee(x, y+CANVASOFFSET, r, Color::rgba(100, 100, 100, 0)); 
                                                orbclient.sync();

                                                dx=evt.x-x;
                                                dy=evt.y-y-CANVASOFFSET;
                                                },
                    EventOption::Button(btn) => {
                                                if btn.left {
                                                    
                                                    let angle = (dx as f32/r as f32).asin();
                                                    
                                                    return Some((r,angle))
                                                    //break 'events 
                                                }
                                                if btn.right{
                                                    break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("Option: {:?}", event_option)}
                                    else{()}
                }
          }
        }
        None
    }

    /// interactive paste 
    fn interact_paste(&mut self, x: i32 , y: i32, opacity: u8, buffer: orbimage::Image, window: &mut Window) {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut w = true;
         let width = buffer.width();
         let height = buffer.height();
         let x = x;
         let y = y;
         let data = buffer.clone().into_data(); //&buffer.clone().into_data()
        'events: loop{
            if w {
                    orbclient.image(x - (width/2) as i32, y + CANVASOFFSET -(height/2) as i32, width, height, &data);
                    orbclient.sync();
                    w = false;
                }
            for event in orbclient.events() { 
                
                match event.to_option() {
                    EventOption::Key(key_event) => {println!("Event:{:?}",key_event); break 'events},
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };

                                                break 'events;
                                                },
                    EventOption::Button(btn) => {
                                                if btn.left {
                                                    //self.image(x, y , width, height, &data); //without transparency
                                                    self.paste_selection(x, y , opacity, buffer);
                                                    break 'events 
                                                }
                                                if btn.right{
                                                    break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("Option: {:?}", event_option)}
                                    else{()}
                }
          }
        }
        
    }
    
    /// draws interactive polyline 
    fn interact_line(&mut self, x: i32 , y: i32, color: Color, width: i32, antialias: bool, window: &mut Window) {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx =0;
         let mut ly =0;
         let mut ox = x;
         let mut oy = y;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => break 'events,
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet.."),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };
                                                if w {
                                                    orbclient.ant_line(ox,
                                                    oy+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    Color::rgba(100, 100, 100, 0),3);//alfa has to be 0 for trick to work
                                                }
                                                w=true;
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                 
                                                orbclient.ant_line(ox,
                                                oy+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                Color::rgba(100, 100, 100, 0),3);//alfa has to be 0 for trick to work
                                                
                                                orbclient.sync();
                                                },
                    EventOption::Button(btn) => {
                                                    if btn.left {
                                                        //quick and dirty workaround to trace thick lines
                                                        //#TODO implement new line method to deal with thickness properly
                                                        for d in 0..width {
                                                            if antialias {
                                                                self.wu_line(ox+d ,oy,lx+d, ly ,color);
                                                            }else{
                                                                self.line(ox+d ,oy,lx+d, ly ,color); //update image
                                                            }
                                                            
                                                            orbclient.line(ox+d ,oy+CANVASOFFSET,lx+d, ly+CANVASOFFSET ,color); //update preview 
                                                        }
                                                        orbclient.sync();
                                                        ox=lx;
                                                        oy=ly;
                                                        w =false;
                                                    }
                                                    if btn.right{
                                                        break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("{:?}", event_option)}
                                    else{ ()}
                }
          }
        }
        
    }
/* fix moved to mainstream orbclient 0.3.9 
    fn mycircle(&mut self, x0: i32, y0: i32, radius: i32, color: Color) {
        let mut x = radius.abs();
        let mut y = 0;
        let mut err = -radius.abs();
        
        match radius {
            radius if radius > 0 => {
                err = 0;
                while x >= y {
                    self.pixel(x0 - x, y0 + y, color);
                    self.pixel(x0 + x, y0 + y, color);
                    self.pixel(x0 - y, y0 + x, color);
                    self.pixel(x0 + y, y0 + x, color);
                    self.pixel(x0 - x, y0 - y, color);
                    self.pixel(x0 + x, y0 - y, color);
                    self.pixel(x0 - y, y0 - x, color);
                    self.pixel(x0 + y, y0 - x, color);
                
                    y += 1;
                    err += 1 + 2*y;
                    if 2*(err-x) + 1 > 0 {
                        x -= 1;
                        err += 1 - 2*x;
                    }
                }      
            },
            
            radius if radius < 0 => {
                while x >= y {
                    let lasty = y;
                    err +=y;
                    y +=1;
                    err += y;
                    self.line4points(x0,y0,x,lasty,color);
                    if err >=0 {
                        if x != lasty{
                           self.line4points(x0,y0,lasty,x,color);
                        }
                        err -= x;
                        x -= 1;
                        err -= x;
                    }
                }

                },
                     _ => {
                            self.pixel(x0, y0, color);
                            
                        },
        }
    }

    fn line4points(&mut self, x0: i32, y0: i32, x: i32, y: i32, color: Color){
        //self.line(x0 - x, y0 + y, (x+x0), y0 + y, color);
        self.rect(x0 - x, y0 + y, x as u32 * 2 + 1, 1, color);
        if y != 0 {
            //self.line(x0 - x, y0 - y, (x+x0), y0-y , color);
            self.rect(x0 - x, y0 - y, x as u32 * 2 + 1, 1, color);
        }
    }
*/
}

pub trait AddOnsToOrbclient {
    fn pixcol(&self, x:i32, y:i32) -> Color;
    fn ant_line(&mut self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color, style: i32);
    fn rect_marquee(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color, style: i32);
    fn circle_marquee(&mut self, x0: i32, y0: i32 , radius: i32 , color: Color);
    fn rect_hollow(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color);
}

impl AddOnsToOrbclient for orbclient::Window{
    ///gets pixel Color at x,y safely
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = (self.width()as i32 * y + x) as usize;
        if p < self.data().len() {
            let rgba = self.data()[p];
            return rgba
        }else { return Color::rgba(0,0,0,0)}
    }
    
    /// Draws ant_line - - -   
    fn ant_line(&mut self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color, style: i32) {
        
        let color = Color::rgba(color.r(),color.g(),color.b(),0);  // make sure alpha is 0 otherwise trick does not work!!
        let mut x = argx1;
        let mut y = argy1;
                
        let dx = if argx1 > argx2 { argx1 - argx2 } else { argx2 - argx1 };
        let dy = if argy1 > argy2 { argy1 - argy2 } else { argy2 - argy1 };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err_tolerance;

        let mut old_color : Color ;
        let mut ct = 0;

        loop {
            if ct == 0 {
            old_color = self.pixcol(x,y);
            // rgb bitwise xor between old and new pixel color
            // New faster implementation xor-ing 32 bit internal color data   
            // Attention :trick does not work as intended xor-ing entire 32bit color data, if new color alpha > 0!!
            self.pixel(x,y,Color{data: (&old_color.data ^ &color.data)}); 
            }
            
            if x == argx2 && y == argy2 { break };

            err_tolerance = 2 * err;

            if err_tolerance > -dx { err -= dy; x += sx; }
            if err_tolerance < dy { err += dx; y += sy; }
            
            if ct<style {ct += 1;}   //3
            else {ct = 0;}            
        }
        //self.sync();
        
    }
    
    ///draws rectangular selection marquee
    fn rect_marquee(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color, style: i32) {
        self.ant_line(argx1,argy1,argx2,argy1,color,style);
        self.ant_line(argx2,argy1,argx2,argy2,color,style);
        self.ant_line(argx2,argy2,argx1,argy2,color,style);
        self.ant_line(argx1,argy2,argx1,argy1,color,style);
        //self.sync();
    }
    
    ///draws hollow rectangle
    fn rect_hollow(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color) {
        self.line(argx1,argy1,argx2,argy1,color);
        self.line(argx2,argy1,argx2,argy2,color);
        self.line(argx2,argy2,argx1,argy2,color);
        self.line(argx1,argy2,argx1,argy1,color);
    }

    fn circle_marquee(&mut self, x0: i32, y0: i32 , radius: i32 , color: Color) {
        let mut x = radius.abs();
        let mut y = 0;
        let mut err = 0;
        let mut old_color : Color ;
        while x >= y {
            old_color = self.pixcol(x0 - x, y0+ y);
            self.pixel(x0 - x, y0 + y, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 + x, y0+ y);
            self.pixel(x0 + x, y0 + y, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 - y, y0+ x);
            self.pixel(x0 - y, y0 + x, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 + y, y0+ x);
            self.pixel(x0 + y, y0 + x, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 - x, y0 - y);
            self.pixel(x0 - x, y0 - y, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 + x, y0 - y);
            self.pixel(x0 + x, y0 - y, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 - y, y0 - x);
            self.pixel(x0 - y, y0 - x, Color{data: (&old_color.data ^ &color.data)});
            old_color = self.pixcol(x0 + y, y0 -x);
            self.pixel(x0 + y, y0 - x, Color{data: (&old_color.data ^ &color.data)});
        
            y += 1;
            err += 1 + 2*y;
            if 2*(err-x) + 1 > 0 {
                x -= 1;
                err += 1 - 2*x;
            }
        } 
    }

}

