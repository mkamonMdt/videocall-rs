use gloo::timers::callback::Interval;
use gloo_timers::callback::Timeout;
use log::debug;
use videocall_client::{CameraEncoder, MicrophoneEncoder, ScreenEncoder, VideoCallClient};

use std::fmt::Debug;
use yew::prelude::*;

use crate::components::device_selector::DeviceSelector;

const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Start,
    EnableScreenShare,
    DisableScreenShare,
    EnableMicrophone(bool),
    DisableMicrophone,
    EnableVideo(bool),
    DisableVideo,
    AudioDeviceChanged(String),
    VideoDeviceChanged(String),
}

pub struct Host {
    pub camera: CameraEncoder,
    pub microphone: MicrophoneEncoder,
    pub screen: ScreenEncoder,
    pub share_screen: bool,
    pub mic_enabled: bool,
    pub video_enabled: bool,
    notification: Option<Interval>,
}

#[derive(Properties, Debug, PartialEq)]
pub struct MeetingProps {
    #[prop_or_default]
    pub id: String,

    pub client: VideoCallClient,

    pub share_screen: bool,

    pub mic_enabled: bool,

    pub video_enabled: bool,
}

impl Host {
    fn update_notification(&mut self, client: &VideoCallClient) {
        if !self.has_ongoing_stream() {
            let _ = self.notification.take();
        } else if self.notification.is_none() {
            let client = client.clone();
            self.notification = Some(Interval::new(1000, move || {
                client.send_stream_notification();
            }));
        }
    }

    fn has_ongoing_stream(&self) -> bool {
        self.video_enabled || self.share_screen
    }
}

impl Component for Host {
    type Message = Msg;
    type Properties = MeetingProps;

    fn create(ctx: &Context<Self>) -> Self {
        let client = &ctx.props().client;
        Self {
            camera: CameraEncoder::new(client.clone(), VIDEO_ELEMENT_ID),
            microphone: MicrophoneEncoder::new(client.clone()),
            screen: ScreenEncoder::new(client.clone()),
            share_screen: ctx.props().share_screen,
            mic_enabled: ctx.props().mic_enabled,
            video_enabled: ctx.props().video_enabled,
            notification: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // Determine if we should start/stop screen share.
        if self.screen.set_enabled(ctx.props().share_screen) && ctx.props().share_screen {
            self.share_screen = ctx.props().share_screen;
            ctx.link().send_message(Msg::EnableScreenShare);
        } else if self.share_screen != ctx.props().share_screen {
            self.share_screen = ctx.props().share_screen;
            ctx.link().send_message(Msg::DisableScreenShare);
        }
        // Determine if we should start/stop microphone.
        if self.microphone.set_enabled(ctx.props().mic_enabled) {
            self.mic_enabled = ctx.props().mic_enabled;
            ctx.link()
                .send_message(Msg::EnableMicrophone(ctx.props().mic_enabled));
        } else if self.mic_enabled != ctx.props().mic_enabled {
            self.mic_enabled = ctx.props().mic_enabled;
            ctx.link().send_message(Msg::DisableMicrophone)
        }
        // Determine if we should start/stop video.
        if self.camera.set_enabled(ctx.props().video_enabled) {
            self.video_enabled = ctx.props().video_enabled;
            ctx.link()
                .send_message(Msg::EnableVideo(ctx.props().video_enabled));
        } else if self.video_enabled != ctx.props().video_enabled {
            self.video_enabled = ctx.props().video_enabled;
            ctx.link().send_message(Msg::DisableVideo)
        }

        self.update_notification(&ctx.props().client);

        if first_render {
            ctx.link().send_message(Msg::Start);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::EnableScreenShare => {
                self.screen.start();
                true
            }
            Msg::DisableScreenShare => {
                self.screen.stop();
                true
            }
            Msg::Start => true,
            Msg::EnableMicrophone(should_enable) => {
                if !should_enable {
                    return true;
                }
                self.microphone.start();
                true
            }
            Msg::DisableMicrophone => {
                self.microphone.stop();
                true
            }
            Msg::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }
                self.camera.start();
                true
            }
            Msg::DisableVideo => {
                self.camera.stop();
                true
            }
            Msg::AudioDeviceChanged(audio) => {
                if self.microphone.select(audio) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::VideoDeviceChanged(video) => {
                if self.camera.select(video) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(true));
                    });
                    timeout.forget();
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);
        html! {
            <>
                <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
            </>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        debug!("destroying");
        self.camera.stop();
        self.microphone.stop();
        self.screen.stop();
    }
}
