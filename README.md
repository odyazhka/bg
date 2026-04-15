# bg
скрипт для управления яркостью

## Скриншоты:
<img width="893" height="1040" alt="изображение" src="https://github.com/user-attachments/assets/69175ffd-b80d-4485-9539-572e32755eae" />
<img width="876" height="1024" alt="изображение" src="https://github.com/user-attachments/assets/c68e0eba-7b31-4db2-867c-c299de949926" />
<img width="882" height="1030" alt="изображение" src="https://github.com/user-attachments/assets/1ed55aeb-b25e-43b8-bb78-369d3212ac36" />
<img width="884" height="1027" alt="изображение" src="https://github.com/user-attachments/assets/ba7d3060-4290-45e9-b9a6-21afedceb060" />
<img width="883" height="1031" alt="изображение" src="https://github.com/user-attachments/assets/0b005d8d-a5f2-4eed-9fbc-3d048f25b130" />


## Установка:

#### 1. Убрать ввод пароля для изменения яркости

Если не интел заменить на intel на amd

```
sudo mkdir -p /etc/udev/rules.d/
sudo nano /etc/udev/rules.d/99-backlight.rules
```

Вставить туда:
```
ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="intel_backlight", RUN+="/bin/chmod 666 /sys/class/backlight/intel_backlight/brightness"
```

Ввести команды в терминале:

```
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo chmod 666 /sys/class/backlight/intel_backlight/brightness
```

Проверить:
```
ls -l /sys/class/backlight/intel_backlight/brightness
```

Если *-rw-rw-rw-* то всё хорошо

#### 2. Переместить bg.sh в домашнюю директорию и сделать исполняемым
```
chmod +x bg.sh
```
##### 3. Для сохранения последней яроксти после завершения работы добавьте в файл автозапуска:
```
tee /sys/class/backlight/intel_backlight/brightness < $HOME/.bg > /dev/null
```
