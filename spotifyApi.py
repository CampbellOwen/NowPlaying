import base64
import pprint
import requests
import time

class Spotify:
    def authenticate(self):
        print ("not right now")

    def make_request(self, url):
        if (int(time.time()) >= self.expiration_time) or not self.access_token:
            self.refresh_auth()

        headers = {'Authorization': f'Bearer {self.access_token}'}
       # print(f'---- [HEADERS] ----')
       # print(headers)
        
        r = requests.get(url, headers=headers)
        if not (r.status_code == 200 or r.status_code == 204):
            print(f"[ERROR][SPOTIFY] -- Request returned status code {r.status_code}")
            print(f'[ERROR][SPOTIFY] -- Request returned {r.text}')
            if r.status_code == 429:
                timeout = int(r.headers['retry-after'])
                print(f"[ERROR][SPOTIFY] -- Rate limited, waiting {timeout} seconds")
                time.sleep(timeout)

            self.refresh_auth()
            r = requests.get(url)
            if not (r.status_code == 200 or r.status_code == 204):
                print(f"[ERROR][SPOTIFY] -- Request returned status code {r.status_code}")
                print(f'[ERROR][SPOTIFY] -- Request returned {r.text}')
                return

        return r.status_code, r
    
    def refresh_auth(self):
        print("[INFO][SPOTIFY] -- Refreshing auth token")
        hashed_client_code = base64.b64encode(f'{self.client_id}:{self.client_secret}'.encode('utf-8'))
        #print(f"[HASHED_CLIENT_CODE] -- {hashed_client_code}")

        payload = { 'grant_type': 'refresh_token', 'refresh_token': self.refresh_token }
        headers = {'Authorization': f'Basic {hashed_client_code.decode("utf-8")}'}

        pp = pprint.PrettyPrinter(indent=4)
        #print(f'---- [PAYLOAD] ----')
        #pp.pprint(payload)
        #print(f'---- [HEADERS] ----')
        #pp.pprint(headers)

        r = requests.post("https://accounts.spotify.com/api/token", data=payload, headers=headers)
        
        if not r.status_code == 200:
            print(f"[ERROR][SPOTIFY] -- Request returned status code {r.status_code}")
            print(f'[ERROR][SPOTIFY] -- Request returned {r.text}')
            return

        result = r.json()
        self.access_token = result['access_token']
        self.expiration_time = int(result['expires_in']) + int(time.time())
        print(f'[INFO][SPOTIFY][ACCESS_TOKEN] -- {self.access_token}')
        print(f'[INFO][SPOTIFY][EXPIRY_TIME] -- {self.expiration_time}')

    def current_song(self):
        code, result = self.make_request("https://api.spotify.com/v1/me/player/currently-playing")
        if not code == 200:
            return None

        result = result.json()

        data = {}
        data['artist'] = result['item']['artists'][0]['name']
        data['album'] = result['item']['album']['name']
        data['release_date'] = result['item']['album']['release_date']
        data['song'] = result['item']['name']
        data['song_popularity'] = result['item']['popularity']
        data['total_tracks'] = result['item']['album']['total_tracks']
        data['track_number'] = result['item']['track_number']

        album_urls = [(image['height'], image['url']) for image in result['item']['album']['images']]
        biggest_image = sorted(album_urls, key=lambda img: img[0], reverse=True)

        data['album_url'] = biggest_image[0][1]
        data['album_image_height'] = int(biggest_image[0][0])

        return data

        
    def __init__(self, client_id, client_secret, refresh_token):
        self.client_id = client_id
        self.client_secret = client_secret
        self.refresh_token = refresh_token
        self.refresh_auth()
