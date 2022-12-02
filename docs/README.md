
# Documentation of the technical architecture of the growbe-mainboard process


The application is running in multiple thread with a task scheduler `Tokio` that
we use to start lightweight task.


The application have multiple long running thread in charge of some part of the application.


We have :

* Comboards tasks : each comboard is running in is own task
