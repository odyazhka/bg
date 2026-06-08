#!/bin/sh

# Имя твоего файла
BIN="bg"

echo "--- Установка управления яркостью (AMD) ---"

# 1. Проверка
if [ ! -f "./$BIN" ]; then
    echo "❌ Ошибка: Файл '$BIN' не найден в этой папке!"
    ls -l
    exit 1
fi

# Определяем настоящего пользователя (работает и без sudo, и с ним)
if [ -n "$SUDO_USER" ]; then
    REAL_USER="$SUDO_USER"
    REAL_HOME="/home/$SUDO_USER"
else
    REAL_USER="$USER"
    REAL_HOME="$HOME"
fi

echo "   Настоящий пользователь: $REAL_USER"
echo "   Домашняя директория:    $REAL_HOME"

# 2. Создаём ~/.local/ для реального пользователя (если нет)
echo "[0/3] Создаю директорию $REAL_HOME/.local/ ..."
mkdir -p "$REAL_HOME/.local"
chown "$REAL_USER:$REAL_USER" "$REAL_HOME/.local" 2>/dev/null || true

# 3. Права и копирование
echo "[1/3] Копирую бинарник в /usr/local/bin/brightness..."
chmod +x "./$BIN"
sudo cp "./$BIN" /usr/local/bin/brightness
sudo chmod +x /usr/local/bin/brightness

# 4. Настройка прав (udev)
echo "[2/3] Создаю правило для работы без sudo..."
echo 'ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="amdgpu_bl*", RUN+="/bin/chmod 666 /sys/class/backlight/%k/brightness"' | sudo tee /etc/udev/rules.d/99-backlight.rules > /dev/null

# 5. Активация
echo "[3/3] Применяю настройки..."
sudo udevadm control --reload-rules
sudo udevadm trigger
# Открываем доступ прямо сейчас (для текущей сессии)
if [ -d "/sys/class/backlight/amdgpu_bl0" ]; then
    sudo chmod 666 /sys/class/backlight/amdgpu_bl0/brightness
elif [ -d "/sys/class/backlight/amdgpu_bl1" ]; then
    sudo chmod 666 /sys/class/backlight/amdgpu_bl1/brightness
fi

echo ""
echo "--- ✅ ВСЁ ГОТОВО! ---"
echo "Яркость сохраняется в: $REAL_HOME/.local/bg"
echo "Запускать: brightness  (без sudo)"
