use either::Either;
use flume::{Receiver, Sender};
use crate::{Exchange, Symbol};

type SingleChannel<D> = (Sender<D>, Receiver<D>);
type MultiChannel<D> = Vec<SingleChannel<D>>;

// #[derive(Debug, Clone)]
// pub struct Broadcast {
//     exchange: Exchange,
//     symbol: Symbol,
//     channel: Either<SingleChannel<RawData>, MultiChannel<RawData>>,
// }
