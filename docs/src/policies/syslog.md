# Syslog

lockc comes with the following policies about access to the kernel message ring
buffer for each policy level:

* **baseline** - not allowed
* **restricted** - not allowed
* **privileged** - allowed

By default, with the **baseline** policy level, checking the kernel logs from
the container is not allowed:

```bash
# docker run -it --rm registry.opensuse.org/opensuse/toolbox:latest
b10f9fa4a385:/ # dmesg
dmesg: read kernel buffer failed: Operation not permitted
```
