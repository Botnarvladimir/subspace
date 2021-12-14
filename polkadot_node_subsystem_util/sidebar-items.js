initSidebarItems({"constant":[["JOB_CHANNEL_CAPACITY","Capacity of channels to and from individual jobs"],["JOB_GRACEFUL_STOP_DURATION","Duration a job will wait after sending a stop signal before hard-aborting."]],"enum":[["Error","Utility errors"],["FromJobCommand","Commands from a job to the broader subsystem."],["JobsError","Error which can be returned by the jobs manager"],["OverseerError","An error type that describes faults that may happen"]],"fn":[["request_extract_bundles","Request `ExtractBundles` from the runtime"],["request_from_runtime","Request some data from the `RuntimeApi`."],["request_pending_head","Request `PendingHead` from the runtime"]],"mod":[["metrics","This module reexports Prometheus types and defines the [`Metrics`] trait."],["reexports","These reexports are required so that external crates can use the `delegated_subsystem` macro properly."]],"struct":[["JobSender","A sender for messages from jobs, as well as commands to the overseer."],["JobSubsystem","A subsystem which wraps jobs."],["JobSubsystemParams","Parameters to a job subsystem."],["Metronome","Create a stream of ticks with a defined cycle duration."],["Timeout","A future that wraps another future with a `Delay` allowing for time-limited futures."]],"trait":[["JobTrait","This trait governs jobs."],["Subsystem","A trait that describes the `Subsystem`s that can run on the `Overseer`."],["TimeoutExt","Extends `Future` to allow time-limited futures."]],"type":[["RuntimeApiReceiver","A type alias for Runtime API receivers."]]});