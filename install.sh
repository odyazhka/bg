#!/bin/bash

# Названия
BINARY_NAME="brightness_manager" # Замени на имя своего файла
UDEV_FILE="99-backlight.rules"

echo "--- Установка $BINARY_NAME ---"

# 1. Проверка наличия бинарника
if [ ! -f "./$BINARY_NAME" ]; then
    echo "Ошибка: Файл $BINARY_NAME не найден в текущей директории!"
    exit 1
fi

# 2. Копируем бинарник в системную папку
echo "[1/3] Копирование бинарника в /usr/local/bin..."
sudo cp "./$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"
sudo chmod +x "/usr/local/bin/$BINARY_NAME"

# 3. Создаем правило udev (чтобы работал без sudo и права не слетали)
echo "[2/3] Создание правила udev для доступа к яркости..."
echo 'ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="intel_backlight", RUN+="/bin/chmod 666 /sys/class/backlight/%k/brightness"' | sudo tee /etc/udev/rules.d/$UDEV_FILE > /dev/null

# 4. Применяем правила прямо сейчас
echo "[3/3] Применение настроек..."
sudo udevadm control --reload-rules
sudo udevadm trigger
# На всякий случай форсируем права на текущую сессию
sudo chmod 666 /sys/class/backlight/intel_backlight/brightness

echo "--- Готово! ---"
echo "Теперь ты можешь запускать программу командой: $BINARY_NAME"
