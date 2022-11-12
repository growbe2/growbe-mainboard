use crate::protos::board::HostInformation;

pub fn get_host_information() -> HostInformation {
	let un = uname::uname().unwrap();
	let mut host = HostInformation::new();

	host.architecture = un.machine;
	host.deviceType = un.nodename;
	host.os = un.sysname;
	host.kernelVersion = un.version;
	host.kernel = un.release;

	return host;
}