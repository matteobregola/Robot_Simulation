pub(crate) mod errors {
    #[derive(Debug)]
    pub(crate) enum Error{
        CollectError,
        LogicError,
    }

    pub(crate) fn terminate_with_error(err:Error) -> ! {
        panic!("Terminate with error{:?}", err);
    }
}
