Bifrost allows lightning nodes exchange direct or onion-wrapped messages.
It defines that such messages are processed by a specific application, i.e.
each message contains a messaging application id, which is a 16-bit integer.
Integers below 0x8000 are reserved for standard apps, each of which should be
standardized by LNP/BP Strandards application, while values above 0x8000 can
be used by independent developers. The id of the bifrost application for short 
is named *bifrost app id* (BAID). It is used not only for messaging, but for 
other bifrost application-specific customization, like channel modifications 
with DLCs etc; thus Bifrost applications are not restricted to messsaging.

Storm protocols provides a messaging application on top of Bifrost, identified
by *bifrost app id* 0x0001. Storm defines a standard of data organization
in bifrost messages, which is used to run arbitrary apps on top providing 
chatting and data storage functionality.

On top of Storm there might be multiple applications processing data structured
by the storm protocol. These applications are called *storm applications* and
are also identified by another 16-bit integer, with the same principle as used
in bifrost messaging applications. This id is called SAID for short.

Thus, Bifrost message will contain two different ids of applications: Bifrost 
application (BAID), which in case of Storm protocol is always 0x0001, and Storm 
application (SAID).
