## Use a hex viewer for raw inspection

If you just want to look at raw bytes:
```
hexdump -C disk.img | less
```

But this is low-level—you won’t easily spot HELLO.TXT unless you search for ASCII strings:
```
strings disk.img | grep HELLO
```


## Mount the image in Linux

If you’re running Linux 

Create a loop device:

```
sudo losetup -Pf disk.img
```

That creates something like /dev/loop0 with partitions /dev/loop0p1, /dev/loop0p2, etc.

Mount the first partition:
```
sudo mkdir /mnt/diskimg
sudo mount /dev/loop0p1 /mnt/diskimg
ls -l /mnt/diskimg
```

You should see HELLO.TXT.

When done:
```
sudo umount /mnt/diskimg
sudo losetup -d /dev/loop0
```
