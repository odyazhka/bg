#!/bin/bash

# Имя твоего файла
BIN="bg"

echo "--- Установка управления яркостью ---"

# 1. Проверка
if [ ! -f "./$BIN" ]; then
    echo "❌ Ошибка: Файл '$BIN' не найден в этой папке!"
    ls -l
    exit 1
fi

# 2. Права и копирование
echo "[1/3] Копирую бинарник в /usr/local/bin/brightness..."
chmod +x "./$BIN"
sudo cp "./$BIN" /usr/local/bin/brightness
sudo chmod +x /usr/local/bin/brightness

# 3. Настройка прав (udev)
echo "[2/3] Создаю правило для работы без sudo..."
echo 'ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="intel_backlight", RUN+="/bin/chmod 666 /sys/class/backlight/%k/brightness"' | sudo tee /etc/udev/rules.d/99-backlight.rules > /dev/null

# 4. Активация
echo "[3/3] Применяю настройки..."
sudo udevadm control --reload-rules
sudo udevadm trigger
# Открываем доступ прямо сейчас (для текущей сессии)
if [ -d "/sys/class/backlight/intel_backlight" ]; then
    sudo chmod 666 /sys/class/backlight/intel_backlight/brightness
fi

echo "--- ✅ ВСЁ ГОТОВО! ---"
echo "Теперь можешь запускать программу просто словом: brightness"
