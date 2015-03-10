use ffi::*;
use bus::Bus;
use util::*;

use libc::c_void;
use std::thread;

unsafe impl Sync for GstElement {}
unsafe impl Send for GstElement {}
unsafe impl Sync for Element {}
unsafe impl Send for Element {}

pub struct Element{
    element: *mut GstElement,
    speed: f64,
    last_pos_ns: i64
}

impl Drop for Element{
	fn drop(&mut self){
		self.set_state(GST_STATE_NULL);
		self.get_state(-1);
		unsafe{
			gst_object_unref(self.gst_element() as *mut c_void);
		}
	}
}

impl Element{
    pub fn new(element_name: &str, name: &str) -> Option<Element>{
        unsafe{
            let element = gst_element_factory_make(to_c_str!(element_name), to_c_str!(name));
            if element != ptr::null_mut::<GstElement>(){
                gst_object_ref_sink(mem::transmute(element));
                Some( Element{element: element, speed: 1.0, last_pos_ns: 0} )
            }else{
				println!("Erroro creating {} return {:?}",element_name, element);
                None
            }
        }
    }

    pub fn factory_make(element: &str, name: &str) -> Option<Element>{
		Element::new(element,name)
	}
    
    pub unsafe fn new_from_gst_element(element: *mut GstElement) -> Option<Element>{
		if element != ptr::null_mut::<GstElement>(){
			Some( Element{element: element, speed: 1.0, last_pos_ns: 0} )
		}else{
			None
		}
    }
    
    pub fn set<T>(&self, name: &str, value: T){
        unsafe{
            g_object_set(self.gst_element() as *mut  c_void, to_c_str!(name), value, ptr::null::<gchar>());
        }
    }
    
}

/// http://gstreamer.freedesktop.org/data/doc/gstreamer/head/gstreamer/html/GstElement.html
pub trait ElementT{
    /// Links this element to dest . 
    /// The link must be from source to destination; the other direction 
    /// will not be tried. The function looks for existing pads that aren't 
    /// linked yet. It will request new pads if necessary. Such pads need 
    /// to be released manually when unlinking. 
    /// If multiple links are possible, only one is established.
	///
	/// Make sure you have added your elements to a bin or pipeline with 
	/// Bin::add() before trying to link them.
	///
	/// returns true if the elements could be linked, false otherwise.
    fn link(&mut self, dst: &mut ElementT) -> bool;
    
    /// Unlinks all source pads of the this element with all sink pads
    /// of the sink element to which they are linked.
	///
	/// If the link has been made using Element::link(), it could have 
	/// created a requestpad, which has to be released using 
	/// gst_element_release_request_pad().
    fn unlink(&mut self, dst: &mut ElementT);
    
	/// Returns the bus of the element. Note that only a Pipeline 
	/// will provide a bus for the application.
    fn bus(&self) -> Option<Bus>;
    
    /// Returns the name of the element
    fn name(&self) -> String;
    
    /// Sets the name of the element
    fn set_name(&mut self, name: &str);
    
    /// Sets the state of the element. This function will try to 
    /// set the requested state by going through all the intermediary 
    /// states and calling the class's state change function for each.
	///
	/// This function can return GST_STATE_CHANGE_ASYNC, in which case 
	/// the element will perform the remainder of the state change 
	/// asynchronously in another thread. An application can use 
	/// get_state() to wait for the completion of the state 
	/// change or it can wait for a GST_MESSAGE_ASYNC_DONE or 
	/// GST_MESSAGE_STATE_CHANGED on the bus.
	///
	/// State changes to GST_STATE_READY or GST_STATE_NULL 
	/// never return GST_STATE_CHANGE_ASYNC.
    fn set_state(&mut self, state: GstState) -> GstStateChangeReturn;
    
    /// Gets the state of the element.
	///
	/// For elements that performed an ASYNC state change, as reported 
	/// by set_state(), this function will block up to the specified 
	/// timeout value for the state change to complete. If the element 
	/// completes the state change or goes into an error, this function 
	/// returns immediately with a return value of GST_STATE_CHANGE_SUCCESS
	/// or GST_STATE_CHANGE_FAILURE respectively.
	///
	/// For elements that did not return GST_STATE_CHANGE_ASYNC, this function 
	/// returns the current and pending state immediately.
	///
	/// This function returns GST_STATE_CHANGE_NO_PREROLL if the element 
	/// successfully changed its state but is not able to provide data yet. 
	/// This mostly happens for live sources that only produce data in 
	/// GST_STATE_PLAYING. While the state change return is equivalent to 
	/// GST_STATE_CHANGE_SUCCESS, it is returned to the application to signal 
	/// that some sink elements might not be able to complete their state change 
	/// because an element is not producing data to complete the preroll. 
	/// When setting the element to playing, the preroll will complete and 
	/// playback will start.
	/// Returns
	///
	/// GST_STATE_CHANGE_SUCCESS if the element has no more pending state and 
	/// the last state change succeeded, GST_STATE_CHANGE_ASYNC if the element 
	/// is still performing a state change or GST_STATE_CHANGE_FAILURE if 
	/// the last state change failed.
    fn get_state(&self, timeout: GstClockTime) -> (GstState, GstState, GstStateChangeReturn);
    
    /// Sends an event to an element. If the element doesn't implement an event
    /// handler, the event will be pushed on a random linked sink pad for 
    /// downstream events or a random linked source pad for upstream events.
	///
	/// This function takes ownership of the provided event so you should
	/// gst_event_ref() it if you want to reuse the event after this call.
    unsafe fn send_event(&mut self, event: *mut GstEvent) -> bool;
    
    /// Simple API to perform a seek on the given element, meaning it just 
    /// seeks to the given position relative to the start of the stream. 
    /// For more complex operations like segment seeks (e.g. for looping) 
    /// or changing the playback rate or seeking relative to the last 
    /// configured playback segment you should use gst_element_seek().
	///
	/// In a completely prerolled PAUSED or PLAYING pipeline, seeking is 
	/// always guaranteed to return TRUE on a seekable media type or FALSE 
	/// when the media type is certainly not seekable (such as a live stream).
	///
	/// Some elements allow for seeking in the READY state, in this case 
	/// they will store the seek event and execute it when they are put to 
	/// PAUSED. If the element supports seek in READY, it will always return
	/// true when it receives the event in the READY state.
    fn seek_simple(&mut self, format: GstFormat, flags: GstSeekFlags, pos: i64) -> bool;
    
    /// Sends a seek event to an element. See [gst_event_new_seek()](http://gstreamer.freedesktop.org/data/doc/gstreamer/head/gstreamer/html/GstEvent.html#gst-event-new-seek)
    /// for the details of the parameters. The seek event is sent to the 
    /// element using send_event().
    fn seek(&mut self, rate: f64, format: GstFormat, flags: GstSeekFlags, start_type: GstSeekType, start: i64, stop_type: GstSeekType, stop: i64) -> bool;
    
    /// Starts a new thread and does the seek in that thread, it also waits
    /// for the seek to finish which makes it less possible that the seek will
    /// fail
    fn seek_async(&mut self, rate: f64, format: GstFormat, flags: GstSeekFlags, start_type: GstSeekType, start: i64, stop_type: GstSeekType, stop: i64);
    
    /// Queries an element (usually top-level pipeline or playbin element) 
    /// for the total stream duration in nanoseconds. This query will only 
    /// work once the pipeline is prerolled (i.e. reached PAUSED or PLAYING 
    /// state). The application will receive an ASYNC_DONE message on the 
    /// pipeline bus when that is the case.
	///
	/// If the duration changes for some reason, you will get a 
	/// DURATION_CHANGED message on the pipeline bus, in which case you should
	/// re-query the duration using this function.
    fn query_duration(&self, format: GstFormat) -> Option<i64>;
    
    /// Queries an element (usually top-level pipeline or playbin element) 
    /// for the stream position in nanoseconds. This will be a value between 0
    /// and the stream duration (if the stream duration is known). This query 
    /// will usually only work once the pipeline is prerolled (i.e. reached 
    /// PAUSED or PLAYING state). The application will receive an ASYNC_DONE 
    /// message on the pipeline bus when that is the case.
    fn query_position(&self, format: GstFormat) -> Option<i64>;
    
    /// Shortcut for query_duration with format == TIME
    fn duration_ns(&self) -> Option<i64>;
    
    /// Shortcut for query_duration with format == TIME and conversion to 
    /// seconds
    fn duration_s(&self) -> Option<f64>;
    
    /// Shortcut for query_position with format == TIME
    fn position_ns(&self) -> i64;
    
    /// Shortcut for query_position with format == TIME and conversion to 
    /// pct as 0..1
    fn position_pct(&self) -> Option<f64>;
    
    /// Shortcut for query_position with format == TIME and conversion to 
    /// seconds
    fn position_s(&self) -> f64;
    
    /// Returns the current playback rate of the element
    fn speed(&self) -> f64;
    
    /// Shortcut for seek to a ceratin position in ns
    fn set_position_ns(&mut self, ns: i64) -> bool;
    
    /// Shortcut for seek to a ceratin position in secs
    fn set_position_s(&mut self, s: f64) -> bool;
    
    /// Shortcut for seek to a ceratin position in pcs as 0..1
    fn set_position_pct(&mut self, pct: f64) -> bool;
    
    /// Shortcut for seek to the current position but change in playback
    /// rate
    fn set_speed(&mut self, speed: f64) -> bool;
    
    /// Shortcut for seek to a ceratin position in ns starting the seek
    /// in a different thread and waiting for it to finish to avoid that 
    /// the seek won't happen
    fn set_position_ns_async(&mut self, ns: i64);
    
    /// Shortcut for seek to a ceratin position in secs starting the seek
    /// in a different thread and waiting for it to finish to avoid that 
    /// the seek won't happen
    fn set_position_s_async(&mut self, s: f64);
    
    /// Shortcut for seek to a ceratin position in pcs as 0..1 starting the
    /// seek in a different thread and waiting for it to finish to avoid that 
    /// the seek won't happen
    fn set_position_pct_async(&mut self, pct: f64) -> bool;
    
    /// Shortcut for seek to the current position but change in playback
    /// rate starting the seek in a different thread and waiting for it to 
    /// finish to avoid that the seek won't happen
    fn set_speed_async(&mut self, speed: f64) -> bool;
    
    // fn set<T>(&self, name: &str, value: T);
    
    /// shortcut to set_state with state == NULL
    fn set_null_state(&mut self);
    
    /// shortcut to set_state with state == READY
    fn set_ready_state(&mut self);
    
    /// shortcut to set_state with state == PAUSED
    fn pause(&mut self);
    
    /// shortcut to set_state with state == PLAYING
    fn play(&mut self);
    
    /// shortcut to query the state and returns state == PAUSED
    fn is_paused(&self) -> bool;
    
    /// shortcut to query the state and returns state == PLAYING
    fn is_playing(&self) -> bool;
    
    /// shortcut to query the state and returns state == NULL
    fn is_null_state(&self) -> bool;
    
    /// shortcut to query the state and returns state == READY
    fn is_ready_state(&self) -> bool;
    
    /// Returns a const raw pointer to the internal GstElement
    unsafe fn gst_element(&self) -> *const GstElement;
    
    /// Returns a mutable raw pointer to the internal GstElement
    unsafe fn gst_element_mut(&mut self) -> *mut GstElement;
}

impl ElementT for Element{
    
    fn link(&mut self, dst: &mut ElementT) -> bool{
        unsafe{
            gst_element_link(self.gst_element_mut(), dst.gst_element_mut()) == 1
        }
    }
    
    fn unlink(&mut self, dst: &mut ElementT){
        unsafe{
            gst_element_unlink(self.gst_element_mut(), dst.gst_element_mut());
        }
    }
    
    fn bus(&self) -> Option<Bus>{
        unsafe{
            Bus::new(gst_element_get_bus(mem::transmute(self.gst_element())),true)
        }
    }
    
    fn name(&self) -> String{
        unsafe{
            let c_str_name = gst_object_get_name(self.gst_element() as *mut GstObject);
            from_c_str!(c_str_name).to_string()
        }
    }
    
    fn set_name(&mut self, name: &str){
        unsafe{
            gst_object_set_name(self.gst_element() as *mut GstObject, to_c_str!(name));
        }
    }
    
    fn set_state(&mut self, state: GstState) -> GstStateChangeReturn{
        unsafe{
            gst_element_set_state(self.gst_element_mut(), state)
        }
    }
    
    fn get_state(&self, timeout: GstClockTime) -> (GstState, GstState, GstStateChangeReturn){
        let mut state: GstState = GST_STATE_NULL;
        let mut pending: GstState = GST_STATE_NULL;
        unsafe{
            let ret = gst_element_get_state(mem::transmute(self.gst_element()), &mut state, &mut pending, timeout);
            (state, pending, ret)
        }
    }
    
    fn send_event(&mut self, event: *mut GstEvent) -> bool{
        unsafe{
            gst_element_send_event(self.gst_element_mut(), event) == 1
        }
    }
    
    fn seek_simple(&mut self, format: GstFormat, flags: GstSeekFlags, pos: i64) -> bool{
        unsafe{
            gst_element_seek_simple(self.gst_element_mut(), format, flags, pos) == 1
        }
    }
    
    fn seek(&mut self, rate: f64, format: GstFormat, flags: GstSeekFlags, start_type: GstSeekType, start: i64, stop_type: GstSeekType, stop: i64) -> bool{
        unsafe{
            gst_element_seek(self.gst_element_mut(), rate, format, flags, start_type, start, stop_type, stop) == 1
        }
    }
    
    fn seek_async(&mut self, rate: f64, format: GstFormat, flags: GstSeekFlags, start_type: GstSeekType, start: i64, stop_type: GstSeekType, stop: i64){
        unsafe{
            let element: u64 = mem::transmute(self.element);
			gst_object_ref(mem::transmute(element));
            thread::spawn(move||{
                let mut state: GstState = GST_STATE_NULL;
                let mut pending: GstState = GST_STATE_NULL;
                gst_element_get_state(mem::transmute(element), &mut state, &mut pending, s_to_ns(1.0));
                gst_element_seek(mem::transmute(element), rate, format, flags, start_type, start, stop_type, stop);
				gst_object_unref(mem::transmute(element));
            });
        }
    }
    
    fn query_duration(&self, format: GstFormat) -> Option<i64>{
        unsafe{
            let mut duration = 0;
            if gst_element_query_duration(mem::transmute(self.gst_element()), format, &mut duration) == 1{
                Some(duration)
            }else{
                None
            }
        }
    }
    
    fn query_position(&self, format: GstFormat) -> Option<i64>{
        unsafe{
            let mut pos = 0;
            if gst_element_query_position(mem::transmute(self.gst_element()), format, &mut pos) == 1{
                Some(pos)
            }else{
                None
            }
        }
    }
    
    fn duration_ns(&self) -> Option<i64>{
        self.query_duration(GST_FORMAT_TIME)
    }
    
    fn duration_s(&self) -> Option<f64>{
        let duration_ns = self.duration_ns();
        match duration_ns{
            Some(t) => Some(ns_to_s(t as u64)),
            None => None
        }
    }
    
    fn position_ns(&self) -> i64{
        match self.query_position(GST_FORMAT_TIME){
            Some(t) => t,
            None => self.last_pos_ns
        }
    }
    
    fn position_pct(&self) -> Option<f64>{
        let pos = self.position_ns();
        let dur = self.duration_ns();
        if dur.is_some(){
            Some( pos as f64 / dur.unwrap() as f64 )
        }else{
            None
        }
    }
    
    fn position_s(&self) -> f64{
        ns_to_s(self.position_ns() as u64)
    }
    
    fn speed(&self) -> f64{
        self.speed
    }
    
    fn set_position_ns(&mut self, ns: i64) -> bool{
        let format = GST_FORMAT_TIME;
	    let flags = GST_SEEK_FLAG_FLUSH; // | GST_SEEK_FLAG_ACCURATE | 
	    let speed = self.speed;
        let ret = if speed > 0.0 {
			self.seek(speed, format,
					flags,
					GST_SEEK_TYPE_SET,
					ns,
					GST_SEEK_TYPE_SET,
					-1)
		} else {
			self.seek(speed, format,
					flags,
					GST_SEEK_TYPE_SET,
					0,
					GST_SEEK_TYPE_SET,
					ns)
		};
        if ret { 
            self.last_pos_ns = ns;
        }
        
        ret
    }
    
    fn set_position_s(&mut self, s: f64) -> bool{
        self.set_position_ns(s_to_ns(s) as i64)
    }
    
    fn set_position_pct(&mut self, pct: f64) -> bool{
        let dur = self.duration_ns();
        match dur{
            Some(t) =>  self.set_position_ns((t as f64 * pct) as i64),
            None => false
        }
    }
    
    fn set_speed(&mut self, speed: f64) -> bool{
        let format = GST_FORMAT_TIME;
	    let flags = GST_SEEK_FLAG_SKIP | GST_SEEK_FLAG_ACCURATE | GST_SEEK_FLAG_FLUSH;
        if speed==0.0 {
            self.speed = speed;
            return self.set_state(GST_STATE_PAUSED) != GST_STATE_CHANGE_FAILURE;
        }
        
        let pos_opt = self.query_position(GST_FORMAT_TIME);
        if pos_opt.is_none(){
            return false;
        }
        
        let pos = pos_opt.unwrap();

        let ret = if speed > 0.0 {
                    self.seek(speed, format,
                            flags,
                            GST_SEEK_TYPE_SET,
                            pos,
                            GST_SEEK_TYPE_SET,
                            -1)
                } else {
                    self.seek(speed, format,
                            flags,
                            GST_SEEK_TYPE_SET,
                            0,
                            GST_SEEK_TYPE_SET,
                            pos)
                };
                
        if ret{
            self.speed = speed;
        }
        
        ret
            
    }
    
    fn set_position_ns_async(&mut self, ns: i64){
        let format = GST_FORMAT_TIME;
	    let flags = GST_SEEK_FLAG_ACCURATE | GST_SEEK_FLAG_FLUSH;
	    let speed = self.speed;
        if speed > 0.0 {
            self.seek_async(speed, format,
                    flags,
                    GST_SEEK_TYPE_SET,
                    ns,
                    GST_SEEK_TYPE_SET,
                    -1);
        } else {
            self.seek_async(speed, format,
                    flags,
                    GST_SEEK_TYPE_SET,
                    0,
                    GST_SEEK_TYPE_SET,
                    ns);
        }
        self.last_pos_ns = ns;
    }
    
    fn set_position_s_async(&mut self, s: f64){
        self.set_position_ns_async(s_to_ns(s) as i64);
    }
    
    fn set_position_pct_async(&mut self, pct: f64) -> bool{
        let dur = self.duration_ns();
        match dur{
            Some(t) =>  {self.set_position_ns_async((t as f64 * pct) as i64); true},
            None => false
        }
    }
    
    fn set_speed_async(&mut self, speed: f64) -> bool{
        let format = GST_FORMAT_TIME;
	    let flags = GST_SEEK_FLAG_SKIP | GST_SEEK_FLAG_ACCURATE | GST_SEEK_FLAG_FLUSH;
        self.speed = speed;
        if speed==0.0 {
            return self.set_state(GST_STATE_PAUSED) != GST_STATE_CHANGE_FAILURE;
        }
        
        let pos_opt = self.query_position(GST_FORMAT_TIME);
        if pos_opt.is_none(){
            return false;
        }
        
        let pos = pos_opt.unwrap();

        if speed > 0.0 {
            self.seek_async(speed, format,
                    flags,
                    GST_SEEK_TYPE_SET,
                    pos,
                    GST_SEEK_TYPE_SET,
                    -1);
            true
        } else {
            self.seek_async(speed, format,
                    flags,
                    GST_SEEK_TYPE_SET,
                    0,
                    GST_SEEK_TYPE_SET,
                    pos);
            true
        }
    }
    
    unsafe fn gst_element(&self) -> *const GstElement{
        self.element
    }
    
    unsafe fn gst_element_mut(&mut self) -> *mut GstElement{
        mem::transmute(self.element)
    }
    
    /*fn set<T>(&self, name: &str, value: T){
        unsafe{
            g_object_set(self.gst_element() as *mut  c_void, name.to_c_str().as_ptr(), value, ptr::null::<gchar>());
        }
    }*/
    
    fn set_null_state(&mut self){
        self.set_state(GST_STATE_NULL);
    }
    
    fn set_ready_state(&mut self){
        self.set_state(GST_STATE_READY);
    }
    
    fn pause(&mut self){
        self.set_state(GST_STATE_PAUSED);
    }
    
    fn play(&mut self){
        self.set_state(GST_STATE_PLAYING);
    }
    
    fn is_paused(&self) -> bool{
        if let (GST_STATE_PAUSED, _pending, GST_STATE_CHANGE_SUCCESS) = self.get_state(-1){
			true
		}else{
			false
		}
    }
    
    fn is_playing(&self) -> bool{
        if let (GST_STATE_PLAYING, _pending, GST_STATE_CHANGE_SUCCESS) = self.get_state(-1){
			true
		}else{
			false
		}
    }
    
    fn is_null_state(&self) -> bool{
        if let (GST_STATE_NULL, _pending, GST_STATE_CHANGE_SUCCESS) = self.get_state(-1){
			true
		}else{
			false
		}
    }
    
    fn is_ready_state(&self) -> bool{
        if let (GST_STATE_READY, _pending, GST_STATE_CHANGE_SUCCESS) = self.get_state(-1){
			true
		}else{
			false
		}
    }
}
