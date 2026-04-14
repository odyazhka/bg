# bg
скрипт для управления яркостью

## Скриншоты:
потом будут

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
