## Tuning

This guide shows options and tricks to gain an optimal performance and resouce
usage.

### Memory usage

Memory usage by lockc depends mostly on BPF maps size. BPF maps are stored in
memory and the biggest BPF maps are the ones related to tracking processes and
containers. Size of those maps depends on the limit of processes (in separate
memory spaces) in the system. That limit is determined by the `kernel.pid_max`
sysctl. By default the limit is 32768. With such limit, memory usage by lockc
should be aproximately 10-20 MB.

If you observe too much memory being used after installing lockc, try to check
the value of `kernel.pid_max`, which can be done with:

```bash
sudo sysctl kernel.pid_max
```

Change of that value (i.e. to 10000) can be done with:

```bash
sudo sysctl kernel.pid_max=10000
```

But that change will be not persistent after reboot. Changing it persistently
requires adding a configuration to `/etc/sysctl.d`. I.e. we could create the
file `/etc/sysctl.d/05-lockc.conf` with the following content:

```
kernel.pid_max = 10000
```

After creating that file, the lower limit is going to be persistent after
reboot.
