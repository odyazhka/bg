#!/bin/bash

# Параметры системы
BRIGHT_FILE="/sys/class/backlight/intel_backlight/brightness"
MAX_BRIGHT=9600
STEP=480
STEP1=96
SAVE_FILE="$HOME/.bg"

# Функция для установки цвета фона и текста
# Аргумент: значение яркости от 0 до 9600
update_visuals() {
    local val=$1
    # Превращаем 0-9600 в диапазон 0-23 (для grayscale в 256-цветной палитре)
    # Коды 232-255 — это оттенки серого (232 - почти черный, 255 - белый)
    local gray_index=$(( val * 23 / MAX_BRIGHT ))
    local color_code=$(( 232 + gray_index ))
    
    # Определяем цвет текста (черный или белый) для контраста
    # Если яркость > 50% (4800), ставим черный текст (0), иначе белый (15)
    local text_color=15
    [[ $val -gt 4800 ]] && text_color=0

    # Очистка экрана и установка цветов
    # \e[48;5;Nm - фон, \e[38;5;Nm - текст
    printf "\e[48;5;%dm\e[38;5;%dm\c" "$color_code" "$text_color"
    clear
    
    local percent=$(( val * 100 / MAX_BRIGHT ))
   # echo "------------------------------------------"
    echo " "
    echo " УПРАВЛЕНИЕ ЯРКОСТЬЮ (↑/↓/←/→/Q)"
   # echo "------------------------------------------"
    echo " "
    echo " Текущее значение: $val ($percent%)"
   # echo " Файл: $BRIGHT_FILE"
   # echo "------------------------------------------"
}

# Читаем текущее значение при старте
CURRENT=$(cat "$BRIGHT_FILE")

# Настройка терминала: скрываем курсор и отключаем эхо
tput civis
trap "tput cnorm; printf '\e[0m'; clear; exit" EXIT

while true; do
    update_visuals "$CURRENT"

    # Чтение 3-х символов для обработки стрелок (они передаются как ESC [ A/B)
    read -rsn1 key
    case "$key" in
        $'\e') # Начало ESC-последовательности
            read -rsn2 -t 0.1 next_key
            if [[ "$next_key" == "[A" ]]; then # ВВЕРХ
                CURRENT=$(( CURRENT + STEP ))
                [[ $CURRENT -le 481 ]] && CURRENT=480
                [[ $CURRENT -gt MAX_BRIGHT ]] && CURRENT=$MAX_BRIGHT

            elif [[ "$next_key" == "[C" ]]; then # ВПРАВО
                CURRENT=$(( CURRENT + STEP1 ))
                [[ $CURRENT -le 97 ]] && CURRENT=96
                [[ $CURRENT -gt MAX_BRIGHT ]] && CURRENT=$MAX_BRIGHT

            elif [[ "$next_key" == "[B" ]]; then # ВНИЗ
                CURRENT=$(( CURRENT - STEP ))
                [[ $CURRENT -le 0 ]] && CURRENT=1
            elif [[ "$next_key" == "[D" ]]; then # ВЛЕВО
                CURRENT=$(( CURRENT - STEP1 ))
                [[ $CURRENT -le 0 ]] && CURRENT=1

            else # Просто нажата клавиша Esc
                exit 0
            fi
            ;;
        q|Q) 
            exit 0
            ;;
    esac

    # Применяем изменения
    echo "$CURRENT" | tee "$BRIGHT_FILE" > /dev/null
    echo "$CURRENT" > "$SAVE_FILE"
done
