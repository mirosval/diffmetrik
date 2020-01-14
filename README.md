# Diffmetrik

CLI Tool that can be repeatedly called to obtain quick information about vital system metrics (CPU Load Average, RAM Usage, Disk and Network usage).

The intended use inside tmux status line.

__NOTE:__ This project is in early stages, not all functionality is implemented. Contributions are welcome.

## Install

```shell
cargo install --git https://github.com/mirosval/diffmetrik
```

## Usage

Now you can call:

```shell
> diffmetrik --metric download
< Not ehough data
> diffmetrik --metric download
> D:  449.32 kB/s
```

First time you will see `Not enough data`, this is because diffmetrik is recording the total amount of bytes transferred over the network at the time of calling. It can only calculate the speed when it is called a second time.

This makes Diffmetrik perfect for environments where it is called often to display some metric. One such example is Tmux status line. For an example configuration you can refer to [my dotfiles](https://github.com/mirosval/dotfiles/blob/master/tmux/tmux.conf.symlink#L87)

`tmux.conf` snippet:

```
# Status
BG1="colour255"
BG2="colour250"
BG3="colour245"
BG4="colour240"
BG5="colour245"

home="#[fg=colour232,bg=$BG1,bold] #S"
user="#[fg=$BG1,bg=$BG2,nobold]#[fg=0,bg=$BG2] #(whoami)"
panels="#[fg=$BG2,bg=$BG3]#[fg=0,bg=$BG3] #I:#P"
datetime="#[fg=$BG3,bg=$BG4]#[fg=0,bg=$BG4] %b %d %H:%M:%S"
end="#[fg=$BG4,bg=colour233,nobold]"

set -g status-left-length 100
set -g status-left "$home $user $panels $datetime $end"

# Set up Diffmetrik
net_speed="#[fg=$BG3,bg=colour233]#[fg=0,bg=$BG3] #(diffmetrik --metric download) #(diffmetrik --metric upload)"
battery="#[fg=$BG2,bg=$BG3]#[fg=0,bg=$BG2] bat: #(~/.dotfiles/scripts/battery.sh)%% "
spotify="#[fg=$BG1,bg=$BG2,bold]#[fg=colour0,bg=$BG1]#(~/.dotfiles/scripts/spotify.sh)"

set -g status-right "$net_speed $battery $spotify"

set -g status-justify centre
```
