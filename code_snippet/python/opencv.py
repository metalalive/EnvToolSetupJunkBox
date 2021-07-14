import template
import numpy, cv2

# Example of cropping and downsizing given image in Numpy array form.

def gen_img_thumbnail(src, new_height, new_width, pos_y=0, pos_x=0):
    # `src` is a numpy nd-array
    old_height, old_width = src.shape[:2]
    assert old_height >= new_height, 'To generate image thumbnail, new height (%s) \
             has to be less than original height (%s)' % (new_height, old_height)
    assert old_width >= new_width  , 'To generate image thumbnail, new width (%s) \
            has to be less than original width (%s)' % (new_width, old_width)
    old_ratio = old_height / old_width
    new_ratio = new_height / new_width
    if old_ratio == new_ratio:
        cropped = src
    else:
        if old_height < old_width:
            # new_height < new_width --> new_ratio < 1, means the width can be extended
            crop_width  = old_height / new_ratio
            crop_height = old_height
        else:
            # new_height < new_width --> new_ratio < 1, means the height can be extended
            crop_width  = old_width
            crop_height = old_width / new_ratio
        pos_y_2 = pos_y - 1 + math.floor(crop_height)
        pos_x_2 = pos_x - 1 + math.floor(crop_width)
        cropped = src[pos_y:pos_y_2 , pos_x:pos_x_2]
    return cv2.resize(cropped, (new_width, new_height), interpolation=cv2.INTER_AREA)

  
def test_gen_img_thumbnail(origin_filepath, codec_ext):
    origin_np = None
    with open(origin_filepath) as origin_file: # TODO, what if it is a huge file ?
        origin_np     = numpy.frombuffer(origin_file.read(), dtype=numpy.uint8)
    if origin_np:
        origin_img    = cv2.imdecode(origin_np, flags=cv2.IMREAD_COLOR)
        thumbnail_img = gen_img_thumbnail(src=origin_img, new_height=height, \
                                          new_width=width, pos_y=0, pos_x=0)
        # `encoded` should be 1-D numpy array
        result, encoded = cv2.imencode('.%s' % codec_ext, thumbnail_img)
        processed_file = tempfile.SpooledTemporaryFile(max_size=2048)
        processed_file.seek(0)
        processed_file.write(encoded.tobytes())
        processed_file.seek(0)
        return processed_file

  
