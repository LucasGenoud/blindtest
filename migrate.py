import json
import sqlite3
import datetime
import os

def dt_to_iso(ts):
    if not ts: return None
    # Assuming ts is in milliseconds
    dt = datetime.datetime.fromtimestamp(ts / 1000.0, tz=datetime.timezone.utc)
    # The rust code expects a valid ISO date, using format like '2023-01-01T00:00:00Z'
    # Actually wait, rust code expects something compatible with strftime('%Y-%m', date)
    # the format 'YYYY-MM-DD HH:MM:SS' or 'YYYY-MM-DDTHH:MM:SSZ' works
    return dt.isoformat().replace("+00:00", "Z")

def init_db(db_path):
    os.makedirs(os.path.dirname(db_path), exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.executescript(open("server/migrations/001_init.sql", encoding="utf-8").read())
    # init canvas with ffffff
    count = conn.execute("SELECT COUNT(*) FROM canvas_pixels").fetchone()[0]
    if count == 0:
        print("Initializing 1,000,000 canvas pixels...")
        def generate_pixels():
            for y in range(1000):
                for x in range(1000):
                    yield (x, y, "ffffff")
        conn.executemany("INSERT INTO canvas_pixels (x, y, color) VALUES (?, ?, ?)", generate_pixels())
    conn.commit()
    conn.close()

def main():
    db_path = "server/data/blindtest.db"
    if not os.path.exists(db_path):
        print(f"DB not found at {db_path}. Initializing...")
        init_db(db_path)

    conn = sqlite3.connect(db_path)
    cur = conn.cursor()

    # 1. users
    print("Migrating users...")
    try:
        users = json.load(open('restore/users.json', encoding='utf-8'))
        for u in users:
            cur.execute("""
                INSERT OR REPLACE INTO users (
                    id, email, name, password, role, clear_mode, hide_carousel,
                    email_confirmation_token, email_confirmed, reset_password_token,
                    reset_password_expires, register_date, deleted
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                u.get('_id'), u.get('email'), u.get('name'), u.get('password'),
                u.get('role', 'user'), 1 if u.get('clearMode') else 0,
                1 if u.get('hideCarousel') else 0, None, 1 if u.get('emailConfirmed') else 0,
                u.get('resetPasswordToken'),
                dt_to_iso(u.get('resetPasswordExpires')),
                dt_to_iso(u.get('registerDate')), 0
            ))
        conn.commit()
        print(f"  Inserted {len(users)} users.")
    except Exception as e:
        print("  Error migrating users:", e)

    # 2. audios
    print("Migrating audios...")
    try:
        audios = json.load(open('restore/audios.json', encoding='utf-8'))
        for a in audios:
            cur.execute("""
                INSERT OR REPLACE INTO audios (
                    id, category, answer, video_url, start_time, superflus,
                    count, submitted_by, added_date, last_updated_by
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                a.get('_id'), a.get('category'), a.get('answer'), a.get('videoUrl'),
                a.get('startTime', 0), 1 if a.get('superflus') else 0,
                a.get('count', 0), a.get('submittedby'), dt_to_iso(a.get('addedDate')),
                a.get('lastUpdatedBy')
            ))
            
            # flagged arrays into flagged_audios
            flagged = a.get('flagged', [])
            if isinstance(flagged, list):
                for uid_obj in flagged:
                    if isinstance(uid_obj, dict):
                        uname = uid_obj.get('user')
                        msg = uid_obj.get('reportMessage', '')
                        cur.execute("SELECT id FROM users WHERE name = ?", (uname,))
                        res = cur.fetchone()
                        uid = res[0] if res else uname
                    else:
                        uid = uid_obj
                        msg = ''
                    if uid:
                        cur.execute("""
                            INSERT OR IGNORE INTO flagged_audios (id, audio_id, user_id, report_message, date)
                            VALUES (?, ?, ?, ?, ?)
                        """, (
                            f"{a.get('_id')}-{uid}", a.get('_id'), uid, msg, dt_to_iso(a.get('addedDate'))
                        ))
        conn.commit()
        print(f"  Inserted {len(audios)} audios.")
    except Exception as e:
        print("  Error migrating audios:", e)

    # 3. ratings
    print("Migrating ratings...")
    try:
        ratings = json.load(open('restore/ratings.json', encoding='utf-8'))
        for r in ratings:
            cur.execute("""
                INSERT OR REPLACE INTO ratings (
                    id, audio_id, user_id, rating, added_date
                ) VALUES (?, ?, ?, ?, ?)
            """, (
                r.get('_id'), r.get('audioId'), r.get('userId'),
                r.get('rating', 0.0), dt_to_iso(r.get('addedDate'))
            ))
        conn.commit()
        print(f"  Inserted {len(ratings)} ratings.")
    except Exception as e:
        print("  Error migrating ratings:", e)
        
    # Recalculate ratings in audios table
    print("Recalculating audio ratings...")
    try:
        cur.execute("""
            UPDATE audios SET 
                rating = (SELECT IFNULL(AVG(rating), 0.0) FROM ratings WHERE audio_id = audios.id),
                rating_count = (SELECT COUNT(*) FROM ratings WHERE audio_id = audios.id)
        """)
        conn.commit()
    except Exception as e:
        print("  Error recalculating audio ratings:", e)

    # 4. custom_blindtests
    print("Migrating custom blindtests...")
    try:
        cbs = json.load(open('restore/customBlindtests.json', encoding='utf-8'))
        for cb in cbs:
            cur.execute("""
                INSERT OR REPLACE INTO custom_blindtests (
                    id, name, public, owner_id, added_date, blindtest_list
                ) VALUES (?, ?, ?, ?, ?, ?)
            """, (
                cb.get('_id'), cb.get('name'), 1 if cb.get('public') else 0,
                cb.get('owner'), dt_to_iso(cb.get('addedDate')),
                json.dumps(cb.get('blindtestList', []))
            ))
        conn.commit()
        print(f"  Inserted {len(cbs)} custom blindtests.")
    except Exception as e:
        print("  Error migrating custom blindtests:", e)

    # 5. suggestions
    print("Migrating suggestions...")
    try:
        suggestions = json.load(open('restore/suggestions.json', encoding='utf-8'))
        for s in suggestions:
            cur.execute("""
                INSERT OR REPLACE INTO suggestions (
                    id, category, answer, video_url, start_time, superflus,
                    submitted_by, added_date
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                s.get('_id'), s.get('category'), s.get('answer'), s.get('videoUrl'),
                s.get('startTime', 0), 1 if s.get('superflus') else 0,
                s.get('submittedby'), dt_to_iso(s.get('addedDate'))
            ))
        conn.commit()
        print(f"  Inserted {len(suggestions)} suggestions.")
    except Exception as e:
        print("  Error migrating suggestions:", e)

    # 6. stats
    print("Migrating stats...")
    try:
        stats = json.load(open('restore/stats.json', encoding='utf-8'))
        for s in stats:
            meta = {k: v for k, v in s.items() if k not in ('_id', 'category', 'user', 'date')}
            cur.execute("""
                INSERT OR REPLACE INTO stats (
                    id, category, user_id, date, metadata
                ) VALUES (?, ?, ?, ?, ?)
            """, (
                s.get('_id'), s.get('category'), s.get('user'),
                dt_to_iso(s.get('date')), json.dumps(meta)
            ))
        conn.commit()
        print(f"  Inserted {len(stats)} stats.")
    except Exception as e:
        print("  Error migrating stats:", e)

    # 7. canvas pixels
    print("Migrating canvas...")
    try:
        canvas = json.load(open('restore/canvas.json', encoding='utf-8'))
        updates = []
        if isinstance(canvas, list):
            for item in canvas:
                if isinstance(item, dict):
                    for k, v in item.items():
                        if isinstance(v, dict) and v.get('c') and v.get('c') != 'ffffff':
                            idx = int(k)
                            x = idx % 1000
                            y = idx // 1000
                            updates.append((v.get('c'), v.get('u'), dt_to_iso(v.get('d')), x, y))
                elif isinstance(item, str) and item != 'ffffff':
                    # Flat array of hex strings if it ever existed
                    idx = canvas.index(item)
                    x = idx % 1000
                    y = idx // 1000
                    updates.append((item, None, None, x, y))
                    
        # Do an UPDATE instead of INSERT OR REPLACE to preserve existing initialized ffffff pixels
        cur.executemany("""
            UPDATE canvas_pixels SET color = ?, user_id = ?, updated_at = ?
            WHERE x = ? AND y = ?
        """, updates)
        conn.commit()
        print(f"  Updated {len(updates)} canvas pixels.")
    except Exception as e:
        print("  Error migrating canvas:", e)

    conn.close()
    print("Migration complete!")

if __name__ == "__main__":
    main()
