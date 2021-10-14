## File access

lockc comes with policies about file access which is based on allow- and
deny-listing. **Baseline** and **restricted** policies have their own pairs of
lists. All those lists should contain path prefixes. All the children of listed
paths/directories are included, since the decision is made by prefix matching.

The deny list has precedence over allow list. That's because main purpose of
the deny list is specifying exceptions whose prefixes are specified in the
allow list, but we don't want to allow them.

To sum it up, when any process in the container tries to access a file, lockc:

1. Checks whether the given path's prefix is in the deny list. If yes, denies
   the access.
2. Checks whether the given path's prefix is in the allow list. If yes, allows
   the access.
3. In case of no matches, denies the access.

By default, the contents of lists are:

* **baseline**
  * allow list
    * */bin*
    * */dev/console*
    * */dev/full*
    * */dev/null*
    * */dev/pts*
    * */dev/tty*
    * */dev/urandom*
    * */dev/zero*
    * */etc*
    * */home*
    * */lib*
    * */proc*
    * */sys/fs/cgroup*
    * */tmp*
    * */usr*
    * */var*
  * deny list
    * */proc/acpi*
* **restricted**
  * allow list
    * */bin*
    * */dev/console*
    * */dev/full*
    * */dev/null*
    * */dev/pts*
    * */dev/tty*
    * */dev/urandom*
    * */dev/zero*
    * */etc*
    * */home*
    * */lib*
    * */proc*
    * */sys/fs/cgroup*
    * */tmp*
    * */usr*
    * */var*
  * deny list
    * */proc/acpi*
    * */proc/sys*
