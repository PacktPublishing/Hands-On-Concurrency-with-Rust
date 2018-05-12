use std::marker::PhantomData;
use event::Event;
use std::sync::mpsc;

pub struct RunnableEmitter<S, EConfig>
where
    S: Send + Emitter<EConfig>,
    EConfig: 'static + Send + Clone,
{
    recv: mpsc::Receiver<Event>,
    state: S,
    config: PhantomData<EConfig>,
}

impl<S, EConfig> RunnableEmitter<S, EConfig>
where
    S: 'static + Send + Emitter<EConfig>,
    EConfig: 'static + Clone + Send,
{
    pub fn new(recv: mpsc::Receiver<Event>, config: EConfig) -> RunnableEmitter<S, EConfig> {
        RunnableEmitter {
            recv: recv,
            state: S::init(config),
            config: PhantomData,
        }
    }
}

pub trait Emitter<EConfig>
where
    Self: 'static + Send + Sized,
    EConfig: 'static + Send + Clone,
{
    ///
    fn new(recv: mpsc::Receiver<Event>, config: EConfig) -> RunnableEmitter<Self, EConfig> {
        RunnableEmitter::<Self, EConfig>::new(recv, config)
    }

    /// Constructs a new emitter.
    fn init(config: EConfig) -> Self;
}
